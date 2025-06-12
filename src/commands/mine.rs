use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateEmbed, EditInteractionResponse,
};
use sqlx::{Database, Pool};
use zayden_core::FormatNum;

use crate::{COIN, MineManager, Mining, Result};

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
