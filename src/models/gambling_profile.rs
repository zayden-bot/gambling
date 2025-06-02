use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use serde::Deserialize;
use serenity::all::{Colour, CreateEmbed, UserId};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool};

use crate::modules::gambling::format_num::FormatNum;
use crate::modules::{
    gambling::shop::ShopItem,
    levels::{LevelsManager, LevelsRow, level_up_xp},
};
use crate::sqlx_lib::TableRow;

use super::{
    COIN, GamblingInventoryManager, GamblingInventoryRow, GamblingInventoryTable, GamblingManager,
    GamblingRow,
};

#[derive(Debug, Default, FromRow)]
pub struct GamblingProfile {
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
    pub inventory: Option<Json<Vec<GamblingItem>>>,
}

impl GamblingProfile {
    pub fn new(id: impl Into<UserId>) -> Self {
        GamblingRow::new(id).into()
    }

    pub async fn from_table(pool: &PgPool, id: impl Into<UserId>) -> sqlx::Result<Self> {
        let id = id.into();

        match Self::main_from_table(pool, id).await {
            Ok(row) => Ok(row),
            Err(_) => Self::fallback_from_table(pool, id).await,
        }
    }

    async fn main_from_table(pool: &PgPool, id: impl Into<UserId>) -> sqlx::Result<Self> {
        let id = id.into();

        sqlx::query_as!(
            GamblingProfile,
            r#"SELECT
                g.id,
                g.cash,
                g.daily,
                g.work,
                g.gift,
                g.game,
                g.diamonds,

                l.total_xp,
                l.last_xp,
                l.xp,
                l.level,
                l.message_count,

                (
                    SELECT COALESCE(jsonb_agg(
                        jsonb_build_object(
                            'quantity', inv.quantity,
                            'item_id', inv.item_id
                        )
                    ), '[]'::jsonb)
                    FROM gambling_inventory inv
                    WHERE inv.user_id = g.id
                ) AS "inventory: Json<Vec<GamblingItem>>"

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

    async fn fallback_from_table(pool: &PgPool, id: impl Into<UserId>) -> sqlx::Result<Self> {
        let id = id.into();

        let gambling_row = GamblingRow::from_table(pool, id).await?;

        let levels_row = LevelsRow::from_table(pool, id).await?;

        let inventory = GamblingInventoryTable::get_user(pool, id).await?;

        Ok(Self {
            id: gambling_row.id,
            cash: gambling_row.coins(),
            daily: gambling_row.daily,
            work: gambling_row.work,
            gift: gambling_row.gift,
            game: gambling_row.game,
            diamonds: gambling_row.gems(),
            total_xp: Some(levels_row.total_xp()),
            last_xp: Some(levels_row.last_xp()),
            xp: Some(levels_row.xp()),
            level: Some(levels_row.level()),
            message_count: Some(levels_row.message_count()),
            inventory: Some(Json(
                inventory.into_iter().map(GamblingItem::from).collect(),
            )),
        })
    }
}

#[async_trait]
impl TableRow for GamblingProfile {
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

        sqlx::query!("DELETE FROM gambling_inventory WHERE user_id = $1", self.id)
            .execute(&mut *tx)
            .await?;

        for item in &self.inventory.as_ref().unwrap().0 {
            sqlx::query!(
                "INSERT INTO gambling_inventory (user_id, item_id, quantity) VALUES ($1, $2, $3)",
                self.id as i64,
                item.item_id,
                item.quantity
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await.unwrap();

        Ok(())
    }
}

impl GamblingManager for GamblingProfile {
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

    fn work_mut(&mut self) -> &mut NaiveDateTime {
        &mut self.work
    }
}

impl GamblingInventoryManager for GamblingProfile {
    fn inventory(&self) -> &[GamblingItem] {
        &self.inventory.as_ref().unwrap().0
    }

    fn inventory_mut(&mut self) -> &mut Vec<GamblingItem> {
        &mut self.inventory.as_mut().unwrap().0
    }
}

impl LevelsManager for GamblingProfile {
    fn level(&self) -> i32 {
        self.level.unwrap_or(1)
    }

    fn total_xp(&self) -> i32 {
        self.total_xp.unwrap_or_default()
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

impl From<GamblingRow> for GamblingProfile {
    fn from(value: GamblingRow) -> Self {
        let levels = LevelsRow::new(value.user_id());

        Self {
            id: value.id,
            cash: value.coins(),
            daily: value.daily,
            work: value.work,
            gift: value.gift,
            game: value.game,
            diamonds: value.gems(),
            total_xp: Some(levels.total_xp()),
            last_xp: Some(levels.last_xp()),
            xp: Some(levels.xp()),
            level: Some(levels.level()),
            message_count: Some(levels.message_count()),
            inventory: Some(Json(Vec::new())),
        }
    }
}

#[derive(Debug, FromRow, Deserialize)]
pub struct GamblingItem {
    pub quantity: i64,
    pub item_id: String,
}

impl From<GamblingInventoryRow> for GamblingItem {
    fn from(value: GamblingInventoryRow) -> Self {
        Self {
            quantity: value.quantity,
            item_id: value.item_id,
        }
    }
}

impl<'a> From<&ShopItem<'a>> for GamblingItem {
    fn from(value: &ShopItem<'a>) -> Self {
        Self {
            quantity: 0,
            item_id: value.id.to_string(),
        }
    }
}
