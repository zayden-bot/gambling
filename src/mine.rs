mod craft;
use std::collections::HashMap;

pub use craft::Craft;

mod dig;
pub use dig::Dig;

use async_trait::async_trait;
use futures::TryStreamExt;
use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateEmbed, EditInteractionResponse,
    ResolvedOption, UserId,
};
use sqlx::{PgExecutor, PgPool, Postgres, prelude::FromRow};
use zayden_core::SlashCommand;

use crate::{Error, Result, cron::CronJob, sqlx_lib::TableRow};

use super::events::{Dispatch, Event, WorkEvent};
use super::{COIN, GamblingManager, GamblingProfile, GamblingRow, ShopCurrency};

const MAX_MINERS_PER_MINE: u8 = 10;
const MAX_MINES_PER_LAND: u8 = 5;
const MAX_LAND_PER_COUNTRY: u8 = 25;
const MAX_COUNTRIES_PER_CONTINENT: u8 = 50;
const MAX_CONTINENTS_PER_PLANT: u8 = 7;
const MAX_PLANTS_PER_SOLAR_SYSTEM: u8 = 8;
const MAX_SOLAR_SYSTEM_PER_GALAXIES: u8 = 100;
const MAX_GALAXIES_PER_UNIVERSE: u8 = u8::MAX;

pub fn register(ctx: &Context) -> [CreateCommand; 3] {
    [
        Mine::register(ctx).unwrap(),
        Dig::register(ctx).unwrap(),
        Craft::register(ctx).unwrap(),
    ]
}

pub struct Mine;

impl Mine {
    pub fn cron_job() -> CronJob {
        CronJob::new("0 0 * * * * *").set_action(async |_, pool| {
            sqlx::query_as!(GamblingMineRow, "SELECT * FROM gambling_mine",)
                .fetch(&pool)
                .try_for_each(|mine_row| {
                    let pool = pool.clone();

                    async move {
                        let mut row = GamblingRow::from_table(&pool, mine_row.id as u64)
                            .await
                            .unwrap();

                        row.add_coins(mine_row.hourly());
                        row.save(&pool).await.unwrap();

                        Ok(())
                    }
                })
                .await
                .unwrap();

            Ok(())
        })
    }
}

#[async_trait]
impl SlashCommand<Error, Postgres> for Mine {
    async fn run(ctx: &Context, interaction: &CommandInteraction, pool: &PgPool) -> Result<()> {
        interaction.defer(ctx).await.unwrap();

        let row = GamblingMineRow::from_table(pool, interaction.user.id)
            .await
            .unwrap();

        let embed = CreateEmbed::new()
            .field(
                "Mine Income",
                format!("{} <:coin:{COIN}> / hour", row.hourly()),
                false,
            )
            .field("Units", row.units(), false);

        interaction
            .edit_response(ctx, EditInteractionResponse::new().embed(embed))
            .await
            .unwrap();

        Ok(())
    }

    fn register(_ctx: &Context) -> Result<CreateCommand> {
        let cmd = CreateCommand::new("mine").description("Show the details of your mine");

        Ok(cmd)
    }
}

pub struct GamblingMineTable;

impl GamblingMineTable {
    async fn get(
        executor: impl PgExecutor<'_>,
        id: impl Into<UserId>,
    ) -> sqlx::Result<Option<GamblingMineRow>> {
        let id = id.into().get();
        sqlx::query_as!(
            GamblingMineRow,
            "SELECT * FROM gambling_mine WHERE id = $1",
            id as i64
        )
        .fetch_optional(executor)
        .await
    }
}

#[derive(FromRow, Default)]
pub struct GamblingMineRow {
    id: i64,
    miners: i64,
    mines: i64,
    land: i64,
    countries: i64,
    continents: i64,
    planets: i64,
    solar_systems: i64,
    galaxies: i64,
    universes: i64,
    prestige: i64,
    coal: i64,
    iron: i64,
    gold: i64,
    redstone: i64,
    lapis: i64,
    diamonds: i64,
    emeralds: i64,
    tech: i64,
    utility: i64,
    production: i64,
}

impl GamblingMineRow {
    pub fn new(id: impl Into<UserId>) -> Self {
        Self {
            id: id.into().get() as i64,
            prestige: 0,
            ..Default::default()
        }
    }

    pub async fn from_table(
        executor: impl PgExecutor<'_>,
        id: impl Into<UserId>,
    ) -> sqlx::Result<Self> {
        let id = id.into();

        GamblingMineTable::get(executor, id)
            .await
            .map(|row| row.unwrap_or_else(|| Self::new(id)))
    }

    pub fn miners_mut(&mut self) -> &mut i64 {
        &mut self.miners
    }

    pub fn mines_mut(&mut self) -> &mut i64 {
        &mut self.mines
    }

    pub fn land_mut(&mut self) -> &mut i64 {
        &mut self.land
    }

    pub fn countries_mut(&mut self) -> &mut i64 {
        &mut self.countries
    }

    pub fn continents_mut(&mut self) -> &mut i64 {
        &mut self.continents
    }

    pub fn planets_mut(&mut self) -> &mut i64 {
        &mut self.planets
    }

    pub fn solar_systems_mut(&mut self) -> &mut i64 {
        &mut self.solar_systems
    }

    pub fn galaxies_mut(&mut self) -> &mut i64 {
        &mut self.galaxies
    }

    pub fn universes_mut(&mut self) -> &mut i64 {
        &mut self.universes
    }

    pub fn coal_mut(&mut self) -> &mut i64 {
        &mut self.coal
    }
    pub fn iron_mut(&mut self) -> &mut i64 {
        &mut self.iron
    }
    pub fn gold_mut(&mut self) -> &mut i64 {
        &mut self.gold
    }
    pub fn redstone_mut(&mut self) -> &mut i64 {
        &mut self.redstone
    }
    pub fn lapis_mut(&mut self) -> &mut i64 {
        &mut self.lapis
    }
    pub fn diamonds_mut(&mut self) -> &mut i64 {
        &mut self.diamonds
    }
    pub fn emeralds_mut(&mut self) -> &mut i64 {
        &mut self.emeralds
    }

    pub fn tech_mut(&mut self) -> &mut i64 {
        &mut self.tech
    }

    pub fn utility_mut(&mut self) -> &mut i64 {
        &mut self.utility
    }

    pub fn production_mut(&mut self) -> &mut i64 {
        &mut self.production
    }

    pub fn hourly(&self) -> i64 {
        if self.miners <= 0 {
            return 0;
        }

        let linear_rate_per_miner = 10;

        self.miners * linear_rate_per_miner
    }

    pub fn max_values(&self) -> HashMap<&str, i64> {
        HashMap::from([
            ("miner", MAX_MINERS_PER_MINE as i64 * (self.mines + 1)),
            ("mine", MAX_MINES_PER_LAND as i64 * (self.land + 1)),
            ("land", MAX_LAND_PER_COUNTRY as i64 * (self.countries + 1)),
            (
                "country",
                MAX_COUNTRIES_PER_CONTINENT as i64 * (self.continents + 1),
            ),
            (
                "continent",
                MAX_CONTINENTS_PER_PLANT as i64 * (self.planets + 1),
            ),
            (
                "planet",
                MAX_PLANTS_PER_SOLAR_SYSTEM as i64 * (self.solar_systems + 1),
            ),
            (
                "solar_system",
                MAX_SOLAR_SYSTEM_PER_GALAXIES as i64 * (self.galaxies + 1),
            ),
            (
                "galaxy",
                MAX_GALAXIES_PER_UNIVERSE as i64 * (self.universes + 1),
            ),
            ("universe", self.prestige + 1),
        ])
    }

    pub fn units(&self) -> String {
        let max_values = self.max_values();

        let max_miners = max_values.get("miner").unwrap();
        let max_mines = max_values.get("mine").unwrap();
        let max_land = max_values.get("land").unwrap();
        let max_countries = max_values.get("country").unwrap();
        let max_continents = max_values.get("continent").unwrap();
        let max_planets = max_values.get("planet").unwrap();
        let max_solar_systems = max_values.get("solar_system").unwrap();
        let max_galaxies = max_values.get("galaxy").unwrap();
        let max_universes = max_values.get("universe").unwrap();

        format!(
            "`{}/{max_miners}` miners
        `{}/{max_mines}` mines
        `{}/{max_land}` plots of land
        `{}/{max_countries}` countries
        `{}/{max_continents}` continents
        `{}/{max_planets}` planets
        `{}/{max_solar_systems}` solar systems
        `{}/{max_galaxies}` galaxies
        `{}/{max_universes}` universes",
            self.miners,
            self.mines,
            self.land,
            self.countries,
            self.continents,
            self.planets,
            self.solar_systems,
            self.galaxies,
            self.universes,
        )
    }
}

#[async_trait]
impl TableRow for GamblingMineRow {
    async fn save(&self, pool: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            "INSERT INTO gambling_mine (
            id,
            miners,
            mines,
            land,
            countries,
            continents,
            planets,
            solar_systems,
            galaxies,
            universes,
            prestige,
            coal,
            iron,
            gold,
            redstone,
            lapis,
            diamonds,
            emeralds,
            tech,
            utility,
            production
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            ON CONFLICT (id)
            DO UPDATE SET miners = EXCLUDED.miners,
            mines = EXCLUDED.mines,
            land = EXCLUDED.land,
            countries = EXCLUDED.countries,
            continents = EXCLUDED.continents,
            planets = EXCLUDED.planets,
            solar_systems = EXCLUDED.solar_systems,
            galaxies = EXCLUDED.galaxies,
            universes = EXCLUDED.universes,
            prestige = EXCLUDED.prestige,
            coal = EXCLUDED.coal,
            iron = EXCLUDED.iron,
            gold = EXCLUDED.gold,
            redstone = EXCLUDED.redstone,
            lapis = EXCLUDED.lapis,
            diamonds = EXCLUDED.diamonds,
            emeralds = EXCLUDED.emeralds,
            tech = EXCLUDED.tech,
            utility = EXCLUDED.utility,
            production = EXCLUDED.production;",
            self.id,
            self.miners,
            self.mines,
            self.land,
            self.countries,
            self.continents,
            self.planets,
            self.solar_systems,
            self.galaxies,
            self.universes,
            self.prestige,
            self.coal,
            self.iron,
            self.gold,
            self.redstone,
            self.lapis,
            self.diamonds,
            self.emeralds,
            self.tech,
            self.utility,
            self.production
        )
        .execute(pool)
        .await.map(|_| ())
    }
}
