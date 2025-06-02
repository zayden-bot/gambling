use serenity::all::UserId;
use sqlx::any::AnyQueryResult;
use sqlx::types::BigDecimal;
use sqlx::{PgExecutor, PgPool};

pub struct GamblingInventoryTable;

impl GamblingInventoryTable {
    pub async fn get_item(
        pool: &PgPool,
        user_id: impl Into<UserId>,
        item_id: &str,
    ) -> sqlx::Result<Option<GamblingInventoryRow>> {
        let user_id = user_id.into().get() as i64;

        sqlx::query_as!(
            GamblingInventoryRow,
            "SELECT * FROM gambling_inventory WHERE user_id = $1 AND item_id = $2",
            user_id,
            item_id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn get_items(
        executor: impl PgExecutor<'_>,
        item_id: &str,
    ) -> sqlx::Result<Vec<GamblingInventoryRow>> {
        sqlx::query_as!(
            GamblingInventoryRow,
            "SELECT * FROM gambling_inventory WHERE item_id = $1",
            item_id
        )
        .fetch_all(executor)
        .await
    }

    pub async fn get_user(
        executor: impl PgExecutor<'_>,
        id: impl Into<UserId>,
    ) -> sqlx::Result<Vec<GamblingInventoryRow>> {
        let id = id.into().get() as i64;

        sqlx::query_as!(
            GamblingInventoryRow,
            "SELECT * FROM gambling_inventory WHERE user_id = $1",
            id
        )
        .fetch_all(executor)
        .await
    }

    pub async fn get_item_count(pool: &PgPool, item_id: &str) -> sqlx::Result<BigDecimal> {
        let data = sqlx::query_scalar!(
            "SELECT SUM(quantity) FROM gambling_inventory WHERE item_id = $1",
            item_id
        )
        .fetch_one(pool)
        .await?;

        Ok(data.unwrap_or_default())
    }

    pub async fn leaderboard(
        pool: &PgPool,
        item_id: &str,
        users: &[i64],
        page: i64,
    ) -> sqlx::Result<Vec<GamblingInventoryRow>> {
        let offset = (page - 1) * 10;

        sqlx::query_as!(
            GamblingInventoryRow,
            "SELECT * FROM gambling_inventory WHERE item_id = $1 AND user_id = ANY($2) ORDER BY quantity DESC LIMIT 10 OFFSET $3",
            item_id,
            users,
            offset
        )
        .fetch_all(pool)
        .await
    }

    pub async fn user_row_number(
        pool: &PgPool,
        user_id: impl Into<UserId>,
        column: &str,
    ) -> sqlx::Result<i64> {
        let user_id = user_id.into().get() as i64;

        let data = sqlx::query_scalar!(
            "SELECT row_number FROM (SELECT user_id, ROW_NUMBER() OVER (ORDER BY $1 DESC) FROM gambling_inventory) AS ranked WHERE user_id = $2",
            column,
            user_id
        )
        .fetch_one(pool)
        .await?;

        Ok(data.unwrap())
    }

    pub async fn delete_item(
        executor: impl PgExecutor<'_>,
        item_id: &str,
    ) -> sqlx::Result<AnyQueryResult> {
        sqlx::query!("DELETE FROM gambling_inventory WHERE item_id = $1", item_id)
            .execute(executor)
            .await
            .map(AnyQueryResult::from)
    }
}

pub struct GamblingInventoryRow {
    pub id: i32,
    pub user_id: i64,
    pub item_id: String,
    pub quantity: i64,
}

impl GamblingInventoryRow {
    pub fn user_id(&self) -> UserId {
        UserId::new(self.user_id as u64)
    }
}
