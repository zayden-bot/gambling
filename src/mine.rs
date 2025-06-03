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
    pub miners: i64,
    pub mines: i64,
    pub land: i64,
    pub countries: i64,
    pub continents: i64,
    pub planets: i64,
    pub solar_systems: i64,
    pub galaxies: i64,
    pub universes: i64,
    pub prestige: i64,
}

impl Mining for MineRow {
    fn miners(&self) -> i64 {
        self.miners
    }

    fn mines(&self) -> i64 {
        self.mines
    }

    fn land(&self) -> i64 {
        self.land
    }

    fn countries(&self) -> i64 {
        self.countries
    }

    fn continents(&self) -> i64 {
        self.continents
    }

    fn planets(&self) -> i64 {
        self.planets
    }

    fn solar_systems(&self) -> i64 {
        self.solar_systems
    }

    fn galaxies(&self) -> i64 {
        self.galaxies
    }

    fn universes(&self) -> i64 {
        self.universes
    }

    fn prestige(&self) -> i64 {
        self.prestige
    }

    fn tech(&self) -> i64 {
        unimplemented!()
    }

    fn utility(&self) -> i64 {
        unimplemented!()
    }

    fn production(&self) -> i64 {
        unimplemented!()
    }

    fn coal(&self) -> i64 {
        unimplemented!()
    }

    fn iron(&self) -> i64 {
        unimplemented!()
    }

    fn gold(&self) -> i64 {
        unimplemented!()
    }

    fn redstone(&self) -> i64 {
        unimplemented!()
    }

    fn lapis(&self) -> i64 {
        unimplemented!()
    }

    fn diamonds(&self) -> i64 {
        unimplemented!()
    }

    fn emeralds(&self) -> i64 {
        unimplemented!()
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
