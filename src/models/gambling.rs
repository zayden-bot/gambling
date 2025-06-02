use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use serenity::all::UserId;
use sqlx::any::AnyQueryResult;
use sqlx::{FromRow, PgExecutor, PgPool};

use crate::modules::gambling::ShopCurrency;
use crate::sqlx_lib::TableRow;
use crate::{Error, Result};

use super::{GamblingManager, START_AMOUNT};

pub struct GamblingTable;

impl GamblingTable {
    pub async fn get(
        executor: impl PgExecutor<'_>,
        id: impl Into<UserId>,
    ) -> sqlx::Result<Option<GamblingRow>> {
        let id = id.into().get() as i64;

        let row = sqlx::query_as!(GamblingRow, "SELECT * FROM gambling WHERE id = $1", id)
            .fetch_optional(executor)
            .await?;

        Ok(row)
    }

    pub async fn user_row_number(
        pool: &PgPool,
        user_id: impl Into<UserId>,
        column: &str,
    ) -> sqlx::Result<i64> {
        let user_id = user_id.into().get() as i64;

        let data = sqlx::query!(
            "SELECT row_number FROM (SELECT id, ROW_NUMBER() OVER (ORDER BY $1 DESC) FROM gambling) AS ranked WHERE id = $2",
            column,
            user_id
        )
        .fetch_one(pool)
        .await?;

        Ok(data.row_number.unwrap())
    }

    pub async fn leaderboard(
        pool: &PgPool,
        users: &[i64],
        column: &str,
        page: i64,
    ) -> sqlx::Result<Vec<GamblingRow>> {
        const LIMIT: i64 = 10;

        let offset = (page - 1) * LIMIT;

        let query = format!(
            "SELECT * FROM gambling WHERE id = ANY($1) ORDER BY {} DESC LIMIT 10 OFFSET $2",
            column
        );

        sqlx::query_as::<_, GamblingRow>(&query)
            .bind(users)
            .bind(offset)
            .fetch_all(pool)
            .await
    }

    pub async fn add_coins(
        pool: &PgPool,
        id: impl Into<UserId>,
        amount: i64,
    ) -> sqlx::Result<AnyQueryResult> {
        let id = id.into().get() as i64;

        sqlx::query!(
            "UPDATE gambling SET cash = cash + $2 WHERE id = $1",
            id,
            amount
        )
        .execute(pool)
        .await
        .map(|r| r.into())
    }

    pub async fn work(pool: &PgPool, id: impl Into<UserId>) -> Result<NaiveDateTime> {
        let id = id.into().get();

        let work = sqlx::query!("SELECT work FROM gambling WHERE id = $1", id as i64)
            .fetch_one(pool)
            .await?
            .work;

        Ok(work)
    }

    pub async fn save(
        executor: impl PgExecutor<'_>,
        row: &GamblingRow,
    ) -> sqlx::Result<AnyQueryResult> {
        sqlx::query!(
            "INSERT INTO gambling (id, cash, daily, work, gift, game, diamonds)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id)
            DO UPDATE SET cash = EXCLUDED.cash,
                          daily = EXCLUDED.daily,
                          work = EXCLUDED.work,
                          gift = EXCLUDED.gift,
                          game = EXCLUDED.game,
                          diamonds = EXCLUDED.diamonds;",
            row.id,
            row.cash,
            row.daily,
            row.work,
            row.gift,
            row.game,
            row.diamonds
        )
        .execute(executor)
        .await
        .map(|r| r.into())
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct GamblingRow {
    pub id: i64,
    cash: i64,
    pub daily: NaiveDate,
    pub work: NaiveDateTime,
    pub gift: NaiveDate,
    pub game: NaiveDateTime,
    diamonds: i64,
}

impl GamblingRow {
    pub fn new(id: impl Into<UserId>) -> Self {
        Self {
            id: id.into().get() as i64,
            cash: START_AMOUNT,
            daily: NaiveDate::default(),
            work: NaiveDateTime::default(),
            gift: NaiveDate::default(),
            game: NaiveDateTime::default(),
            diamonds: 0,
        }
    }

    pub async fn from_table(
        executor: impl PgExecutor<'_>,
        id: impl Into<UserId>,
    ) -> sqlx::Result<Self> {
        let id = id.into();

        GamblingTable::get(executor, id)
            .await
            .map(|row| row.unwrap_or_else(|| Self::new(id)))
    }

    pub fn verify_bet(&self, bet: i64, min: i64) -> Result<()> {
        if bet < min {
            return Err(Error::MinimumBetAmount(min));
        }

        if bet > self.coins() {
            return Err(Error::InsufficientFunds {
                required: bet - self.coins(),
                currency: ShopCurrency::Coins,
            });
        }

        Ok(())
    }
}

#[async_trait]
impl TableRow for GamblingRow {
    async fn save(&self, pool: &PgPool) -> sqlx::Result<()> {
        GamblingTable::save(pool, self).await?;

        Ok(())
    }
}

impl GamblingManager for GamblingRow {
    fn user_id(&self) -> UserId {
        UserId::new(self.id as u64)
    }

    fn coins(&self) -> i64 {
        self.cash
    }

    fn coins_mut(&mut self) -> &mut i64 {
        &mut self.cash
    }

    fn gems(&self) -> i64 {
        self.diamonds
    }

    fn gems_mut(&mut self) -> &mut i64 {
        &mut self.diamonds
    }

    fn game(&self) -> NaiveDateTime {
        self.game
    }

    fn update_game(&mut self) {
        self.game = Utc::now().naive_utc()
    }

    fn work(&self) -> NaiveDateTime {
        self.work
    }

    fn work_mut(&mut self) -> &mut NaiveDateTime {
        &mut self.work
    }
}
