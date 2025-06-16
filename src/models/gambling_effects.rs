use async_trait::async_trait;
use chrono::{NaiveDateTime, Utc};
use serenity::all::UserId;
use sqlx::{Database, Pool, any::AnyQueryResult};

use crate::shop::{LUCKY_CHIP, SHOP_ITEMS, ShopItem};

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

    async fn payout(
        pool: &Pool<Db>,
        user_id: impl Into<UserId> + Send,
        bet: i64,
        mut payout: i64,
    ) -> i64 {
        let user_id = user_id.into();

        let mut tx = pool.begin().await.unwrap();
        let effects = Self::get_effects(&mut *tx, user_id).await.unwrap();

        let mut accumulated_payout = 0;

        let now_naive_utc = Utc::now().naive_utc();

        for effect in effects {
            match effect.expiry {
                None => {
                    Self::remove_effect(&mut *tx, effect.id).await.unwrap();
                }
                Some(expiry_time) => {
                    if expiry_time < now_naive_utc {
                        Self::remove_effect(&mut *tx, effect.id).await.unwrap();
                        continue;
                    }
                }
            }

            let item = SHOP_ITEMS.get(&effect.item_id).unwrap();

            if (effect.item_id.starts_with("payout") || effect.item_id.starts_with("profit"))
                && payout > 0
            {
                accumulated_payout += (item.effect_fn)(bet, payout);
                continue;
            }

            payout = accumulated_payout;

            if item.id == LUCKY_CHIP.id && payout == 0 {
                payout = (item.effect_fn)(bet, payout)
            }
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
