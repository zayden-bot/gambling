use async_trait::async_trait;
use chrono::{NaiveDateTime, Utc};
use serenity::all::UserId;
use sqlx::{Database, Pool, any::AnyQueryResult};

use crate::shop::{SHOP_ITEMS, ShopItem};

#[async_trait]
pub trait EffectsManager<Db: Database> {
    async fn get_effects(
        conn: &mut Db::Connection,
        user_id: impl Into<UserId> + Send,
    ) -> sqlx::Result<Vec<EffectsRow>>;

    async fn add_effect(
        conn: &mut Db::Connection,
        user_id: impl Into<UserId> + Send,
        item: &ShopItem<'_>,
    ) -> sqlx::Result<AnyQueryResult>;

    async fn remove_effect(conn: &mut Db::Connection, id: i32) -> sqlx::Result<AnyQueryResult>;

    async fn payout(pool: &Pool<Db>, user_id: impl Into<UserId> + Send, mut payout: i64) -> i64 {
        let mut tx = pool.begin().await.unwrap();

        let effects = Self::get_effects(&mut *tx, user_id).await.unwrap();

        for effect in effects {
            if effect
                .expiry
                .is_none_or(|expiry| expiry < Utc::now().naive_utc())
            {
                Self::remove_effect(&mut *tx, effect.id).await.unwrap();
            }

            let item = SHOP_ITEMS.get(&effect.item_id).unwrap();

            payout = (item.effect_fn)(payout)
        }

        tx.commit().await.unwrap();

        payout
    }
}

pub struct EffectsRow {
    pub id: i32,
    pub item_id: String,
    pub expiry: Option<NaiveDateTime>,
}
