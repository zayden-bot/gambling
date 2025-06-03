use async_trait::async_trait;
use chrono::{Days, NaiveDate, NaiveTime, Utc};
use serenity::all::{
    Colour, CommandInteraction, Context, CreateCommand, CreateEmbed, EditInteractionResponse,
    UserId,
};
use sqlx::{Database, Pool, any::AnyQueryResult, prelude::FromRow};
use zayden_core::FormatNum;

use crate::{COIN, Coins, Error, Result, START_AMOUNT};

use super::Commands;

#[async_trait]
pub trait DailyManager<Db: Database> {
    async fn row(pool: &Pool<Db>, id: impl Into<UserId> + Send) -> sqlx::Result<Option<DailyRow>>;

    async fn save(pool: &Pool<Db>, row: DailyRow) -> sqlx::Result<AnyQueryResult>;
}

#[derive(FromRow)]
pub struct DailyRow {
    pub id: i64,
    pub coins: i64,
    pub daily: NaiveDate,
}

impl DailyRow {
    pub fn new(id: impl Into<UserId>) -> Self {
        let id = id.into();

        Self {
            id: id.get() as i64,
            coins: 0,
            daily: NaiveDate::default(),
        }
    }
}

impl Coins for DailyRow {
    fn coins(&self) -> i64 {
        self.coins
    }

    fn coins_mut(&mut self) -> &mut i64 {
        &mut self.coins
    }
}

impl Commands {
    pub async fn daily<Db: Database, Manager: DailyManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer(ctx).await.unwrap();

        let mut row = Manager::row(pool, interaction.user.id)
            .await
            .unwrap()
            .unwrap_or_else(|| DailyRow::new(interaction.user.id));

        let now = Utc::now();
        let today = now.naive_utc().date();
        let tomorrow = now
            .with_time(NaiveTime::MIN)
            .unwrap()
            .checked_add_days(Days::new(1))
            .unwrap();

        if row.daily == today {
            return Err(Error::DailyClaimed(tomorrow.timestamp()));
        }

        *row.coins_mut() += START_AMOUNT;
        row.daily = today;

        Manager::save(pool, row).await.unwrap();

        let embed = CreateEmbed::new()
            .description(format!(
                "Collected {} <:coin:{COIN}>",
                START_AMOUNT.format()
            ))
            .colour(Colour::GOLD);

        interaction
            .edit_response(ctx, EditInteractionResponse::new().embed(embed))
            .await
            .unwrap();

        Ok(())
    }

    pub fn register_daily() -> CreateCommand {
        CreateCommand::new("daily").description("Collect your daily coins")
    }
}
