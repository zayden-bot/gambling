use async_trait::async_trait;
use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateEmbed, EditInteractionResponse, UserId,
};
use sqlx::{Database, FromRow, Pool};
use zayden_core::FormatNum;

use crate::{COIN, MineHourly, Mining, Result};

#[async_trait]
pub trait MineManager<Db: Database> {
    async fn row(pool: &Pool<Db>, id: impl Into<UserId> + Send) -> sqlx::Result<Option<MineRow>>;
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

impl MineHourly for MineRow {
    fn miners(&self) -> i64 {
        self.miners
    }

    fn prestige(&self) -> i64 {
        self.prestige
    }
}

use super::Commands;

impl Commands {
    pub async fn mine<Db: Database, Manager: MineManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer(ctx).await?;

        let row = Manager::row(pool, interaction.user.id)
            .await
            .unwrap()
            .unwrap_or_default();

        let embed = CreateEmbed::new()
            .field(
                "Mine Income",
                format!("{} <:coin:{COIN}> / hour", row.hourly().format()),
                false,
            )
            .field("Units", row.units(), false);

        interaction
            .edit_response(ctx, EditInteractionResponse::new().embed(embed))
            .await
            .unwrap();

        Ok(())
    }

    pub fn register_mine() -> CreateCommand {
        CreateCommand::new("mine").description("Show the details of your mine")
    }
}
