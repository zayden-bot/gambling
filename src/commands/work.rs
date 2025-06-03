use async_trait::async_trait;
use serenity::all::{
    Colour, CommandInteraction, Context, CreateCommand, CreateEmbed, EditInteractionResponse,
    UserId,
};
use sqlx::any::AnyQueryResult;
use sqlx::prelude::FromRow;
use sqlx::{Database, Pool};

use crate::events::{Dispatch, Event};
use crate::{COIN, Coins, Gems, GoalsManager, MaxBet, Result, Stamina, StaminaManager};

use super::Commands;

#[derive(FromRow)]
pub struct WorkRow {
    pub id: i64,
    pub coins: i64,
    pub gems: i64,
    pub stamina: i32,
    pub level: i32,
}

impl WorkRow {
    fn new(id: impl Into<UserId>) -> Self {
        let id = id.into();

        Self {
            id: id.get() as i64,
            coins: 0,
            gems: 0,
            stamina: 0,
            level: 0,
        }
    }
}

impl Coins for WorkRow {
    fn coins(&self) -> i64 {
        self.coins
    }

    fn coins_mut(&mut self) -> &mut i64 {
        &mut self.coins
    }
}

impl Gems for WorkRow {
    fn gems(&self) -> i64 {
        self.gems
    }

    fn gems_mut(&mut self) -> &mut i64 {
        &mut self.gems
    }
}

impl Stamina for WorkRow {
    fn stamina(&self) -> i32 {
        self.stamina
    }

    fn stamina_mut(&mut self) -> &mut i32 {
        &mut self.stamina
    }
}

impl MaxBet for WorkRow {
    fn level(&self) -> i32 {
        self.level
    }
}

#[async_trait]
pub trait WorkManager<Db: Database> {
    async fn row(pool: &Pool<Db>, id: impl Into<UserId> + Send) -> sqlx::Result<Option<WorkRow>>;

    async fn save(pool: &Pool<Db>, row: WorkRow) -> sqlx::Result<AnyQueryResult>;
}

impl Commands {
    pub async fn work<
        Db: Database,
        StaminaHandler: StaminaManager<Db>,
        GoalHandler: GoalsManager<Db>,
        WorkHandler: WorkManager<Db>,
    >(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer(ctx).await.unwrap();

        let mut row = match WorkHandler::row(pool, interaction.user.id).await.unwrap() {
            Some(row) => row,
            None => WorkRow::new(interaction.user.id),
        };

        row.verify_work::<Db, StaminaHandler>()?;

        let amount = rand::random_range(100..=500);
        *row.coins_mut() += amount;

        let gem_desc = if rand::random_bool(1.0 / 150.0) {
            row.add_gems(1);
            "\n💎 You found a GEM!"
        } else {
            ""
        };

        let coins = row.coins_str();

        Dispatch::<Db, GoalHandler>::new(pool)
            .fire(&mut row, Event::Work(interaction.user.id))
            .await?;

        row.done_work();

        WorkHandler::save(pool, row).await.unwrap();

        let embed = CreateEmbed::new()
            .description(format!(
                "Collected {amount} <:coin:{COIN}> for working{gem_desc}\nYour coins: {coins}"
            ))
            .colour(Colour::GOLD);

        interaction
            .edit_response(ctx, EditInteractionResponse::new().embed(embed))
            .await
            .unwrap();

        Ok(())
    }

    pub fn register_work() -> CreateCommand {
        CreateCommand::new("work").description("Do some work and get some quick coins")
    }
}
