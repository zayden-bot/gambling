use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use serenity::all::UserId;
use sqlx::PgPool;

use crate::modules::levels::{LevelsManager, LevelsRow};
use crate::sqlx_lib::TableRow;

use super::{GamblingManager, GamblingRow};

#[derive(Clone, Copy)]
pub struct GamblingAndLevel {
    pub id: i64,
    cash: i64,
    pub daily: NaiveDate,
    pub work: NaiveDateTime,
    pub gift: NaiveDate,
    pub game: NaiveDateTime,
    diamonds: i64,
    total_xp: Option<i32>,
    last_xp: Option<NaiveDateTime>,
    xp: Option<i32>,
    level: Option<i32>,
    message_count: Option<i32>,
}

impl GamblingAndLevel {
    pub fn new(id: impl Into<UserId>) -> Self {
        GamblingRow::new(id).into()
    }

    pub async fn from_table(pool: &PgPool, id: impl Into<UserId>) -> sqlx::Result<Self> {
        let id = id.into();

        sqlx::query_as!(
            GamblingAndLevel,
            r#"SELECT
                g.id,
                g.cash,
                g.daily,
                g.work,
                g.gift,
                g.game,
                g.diamonds,

                COALESCE(l.total_xp, 0) as total_xp,
                COALESCE(l.last_xp, '1970-01-01T00:00:00Z') as last_xp,
                COALESCE(l.xp, 0) as xp,
                COALESCE(l.level, 1) as level,
                COALESCE(l.message_count, 0) as message_count

            FROM
                gambling g
            LEFT JOIN
                levels l ON g.id = l.id
            WHERE
                g.id = $1
            GROUP BY
                g.id, l.id;"#,
            id.get() as i64
        )
        .fetch_optional(pool)
        .await
        .map(|row| row.unwrap_or_else(|| Self::new(id)))
    }
}

#[async_trait]
impl TableRow for GamblingAndLevel {
    async fn save(&self, pool: &PgPool) -> sqlx::Result<()> {
        let mut tx = pool.begin().await.unwrap();

        sqlx::query!(
            "INSERT INTO gambling (id, cash, daily, work, gift, game, diamonds)
                   VALUES ($1, $2, $3, $4, $5, $6, $7)
                   ON CONFLICT (id) DO UPDATE SET
                       cash = EXCLUDED.cash, daily = EXCLUDED.daily, work = EXCLUDED.work,
                       gift = EXCLUDED.gift, game = EXCLUDED.game, diamonds = EXCLUDED.diamonds;",
            self.id,
            self.cash,
            self.daily,
            self.work,
            self.gift,
            self.game,
            self.diamonds
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT INTO levels (id, total_xp, last_xp, xp, level, message_count)
                   VALUES ($1, $2, $3, $4, $5, $6)
                   ON CONFLICT (id) DO UPDATE SET
                       total_xp = EXCLUDED.total_xp, last_xp = EXCLUDED.last_xp, xp = EXCLUDED.xp,
                       level = EXCLUDED.level, message_count = EXCLUDED.message_count;",
            self.id,
            self.total_xp,
            self.last_xp,
            self.xp,
            self.level,
            self.message_count
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await.unwrap();

        Ok(())
    }
}

impl GamblingManager for GamblingAndLevel {
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
        self.game = Utc::now().naive_utc();
    }

    fn work(&self) -> NaiveDateTime {
        self.work
    }

    fn work_mut(&mut self) -> &mut NaiveDateTime {
        &mut self.work
    }
}

impl LevelsManager for GamblingAndLevel {
    fn level(&self) -> i32 {
        self.level.unwrap_or(1)
    }

    fn total_xp(&self) -> i32 {
        self.xp.unwrap_or_default()
    }

    fn last_xp(&self) -> NaiveDateTime {
        self.last_xp.unwrap_or_default()
    }

    fn xp(&self) -> i32 {
        self.xp.unwrap_or_default()
    }

    fn message_count(&self) -> i32 {
        self.message_count.unwrap_or_default()
    }
}

impl From<GamblingRow> for GamblingAndLevel {
    fn from(value: GamblingRow) -> Self {
        let level = LevelsRow::new(value.id as u64);

        Self {
            id: value.id,
            cash: value.coins(),
            daily: value.daily,
            work: value.work,
            gift: value.gift,
            game: value.game,
            diamonds: value.gems(),
            total_xp: Some(level.total_xp()),
            last_xp: Some(level.last_xp()),
            xp: Some(level.xp()),
            level: Some(level.level()),
            message_count: Some(level.message_count()),
        }
    }
}
