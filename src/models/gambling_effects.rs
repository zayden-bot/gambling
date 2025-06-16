use std::collections::HashMap;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use serenity::all::UserId;
use sqlx::{Database, Pool, any::AnyQueryResult};

use crate::shop::{LUCKY_CHIP, SHOP_ITEMS, ShopItem};

#[async_trait]
pub trait EffectsManager<Db: Database> {
    async fn get_effects(
        conn: &mut Db::Connection,
        user_id: impl Into<UserId> + Send,
    ) -> sqlx::Result<HashMap<String, i32>>;

    async fn add_effect(
        conn: &mut Db::Connection,
        user_id: impl Into<UserId> + Send,
        item: &ShopItem<'_>,
    ) -> sqlx::Result<AnyQueryResult>;

    async fn remove_effect(conn: &mut Db::Connection, id: i32) -> sqlx::Result<AnyQueryResult>;

    async fn payout(
        pool: &Pool<Db>,
        user_id: impl Into<UserId> + Send,
        bet: i64,
        mut payout: i64,
        win: bool,
    ) -> i64 {
        let base_payout = payout;
        payout = 0;

        let user_id = user_id.into();

        let mut tx = pool.begin().await.unwrap();
        let mut effects = Self::get_effects(&mut *tx, user_id).await.unwrap();

        {
            let lucky_chip = effects.remove(LUCKY_CHIP.id);
            if let Some(id) = lucky_chip {
                Self::remove_effect(&mut *tx, id).await.unwrap();

                if !win {
                    payout = bet;
                }
            }
        }

        for (item_id, id) in effects.drain() {
            Self::remove_effect(&mut *tx, id).await.unwrap();

            let item = SHOP_ITEMS.get(&item_id).unwrap();

            if win && item_id.starts_with("payout") {
                payout += (item.effect_fn)(bet, base_payout);
                continue;
            }
        }

        tx.commit().await.unwrap();

        payout.max(base_payout)
    }
}

pub struct EffectsRow {
    pub id: i32,
    pub item_id: String,
    pub expiry: Option<NaiveDateTime>,
}
