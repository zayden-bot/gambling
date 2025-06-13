use async_trait::async_trait;
use chrono::{NaiveDateTime, Timelike, Utc};
use serenity::all::{
    Colour, CommandInteraction, Context, CreateCommand, CreateEmbed, EditInteractionResponse,
    UserId,
};
use sqlx::any::AnyQueryResult;
use sqlx::prelude::FromRow;
use sqlx::{Database, Pool};
use zayden_core::FormatNum;

use crate::events::{Dispatch, Event};
use crate::{COIN, Coins, Gems, GoalsManager, MaxBet, MineHourly, Result, Stamina, StaminaManager};

use super::Commands;

#[derive(FromRow)]
pub struct WorkRow {
    pub id: i64,
    pub coins: i64,
    pub gems: i64,
    pub stamina: i32,
    pub level: Option<i32>,
    pub miners: Option<i64>,
    pub prestige: Option<i64>,
    pub mine_activity: Option<NaiveDateTime>,
}

impl WorkRow {
    fn new(id: impl Into<UserId>) -> Self {
        let id = id.into();

        Self {
            id: id.get() as i64,
            coins: 0,
            gems: 0,
            stamina: 0,
            level: Some(0),
            miners: Some(0),
            prestige: Some(0),
            mine_activity: Some(Utc::now().naive_utc()),
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
    fn prestige(&self) -> i64 {
        self.prestige.unwrap_or_default()
    }

    fn level(&self) -> i32 {
        self.level.unwrap_or_default()
    }
}

impl MineHourly for WorkRow {
    fn miners(&self) -> i64 {
        self.miners.unwrap_or_default()
    }

    fn prestige(&self) -> i64 {
        self.prestige.unwrap_or_default()
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
        interaction.defer(ctx).await?;

        let mut row = match WorkHandler::row(pool, interaction.user.id).await.unwrap() {
            Some(row) => row,
            None => WorkRow::new(interaction.user.id),
        };

        row.verify_work::<Db, StaminaHandler>()?;

        let base_amount = rand::random_range(100..=500);
        let mine_amount = mine_amount(&row);
        let total_amount = base_amount + mine_amount;

        *row.coins_mut() += total_amount;

        let gem_desc = if rand::random_bool(1.0 / 200.0) {
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
        row.mine_activity = Some(Utc::now().naive_utc());

        let stamina = row.stamina_str();

        WorkHandler::save(pool, row).await.unwrap();

        let embed = CreateEmbed::new()
            .description(format!(
                "Collected {} <:coin:{COIN}> for working{gem_desc}\nYour coins: {coins}\nStamina: {stamina}", total_amount.format()
            ))
            .colour(Colour::GOLD);

        interaction
            .edit_response(ctx, EditInteractionResponse::new().embed(embed))
            .await?;

        Ok(())
    }

    pub fn register_work() -> CreateCommand {
        CreateCommand::new("work").description("Do some work and get some quick coins")
    }
}

fn mine_amount(row: &WorkRow) -> i64 {
    let mine_activity = match row.mine_activity {
        Some(dt) => dt,
        None => {
            return 0;
        }
    };

    let mine_hour = mine_activity
        .date()
        .and_hms_opt(mine_activity.hour(), 0, 0)
        .unwrap()
        .and_utc();

    let duration = Utc::now() - mine_hour;

    duration.num_hours() * row.hourly()
}
