use async_trait::async_trait;
use serenity::all::UserId;
use sqlx::{Database, Pool, any::AnyQueryResult, prelude::FromRow};
use zayden_core::CronJob;

use crate::Mining;

#[async_trait]
pub trait MineManager<Db: Database> {
    async fn row(pool: &Pool<Db>, id: impl Into<UserId> + Send) -> sqlx::Result<Option<MineRow>>;

    async fn add_coins(pool: &Pool<Db>) -> sqlx::Result<AnyQueryResult>;
}

#[derive(Default, FromRow)]
pub struct MineRow {
    miners: i64,
}

impl Mining for MineRow {
    fn miners(&self) -> i64 {
        self.miners
    }

    fn mines(&self) -> i64 {
        todo!()
    }

    fn land(&self) -> i64 {
        todo!()
    }

    fn countries(&self) -> i64 {
        todo!()
    }

    fn continents(&self) -> i64 {
        todo!()
    }

    fn planets(&self) -> i64 {
        todo!()
    }

    fn solar_systems(&self) -> i64 {
        todo!()
    }

    fn galaxies(&self) -> i64 {
        todo!()
    }

    fn universes(&self) -> i64 {
        todo!()
    }

    fn prestige(&self) -> i64 {
        todo!()
    }

    fn tech(&self) -> i64 {
        todo!()
    }

    fn utility(&self) -> i64 {
        todo!()
    }

    fn production(&self) -> i64 {
        todo!()
    }

    fn coal(&self) -> i64 {
        todo!()
    }

    fn iron(&self) -> i64 {
        todo!()
    }

    fn gold(&self) -> i64 {
        todo!()
    }

    fn redstone(&self) -> i64 {
        todo!()
    }

    fn lapis(&self) -> i64 {
        todo!()
    }

    fn diamonds(&self) -> i64 {
        todo!()
    }

    fn emeralds(&self) -> i64 {
        todo!()
    }
}

pub struct Mine;

impl Mine {
    pub fn cron_job<Db: Database, Manager: MineManager<Db>>() -> CronJob<Db> {
        CronJob::new("0 0 * * * * *").set_action(|_, pool| async move {
            Manager::add_coins(&pool).await.unwrap();
        })
    }
}
