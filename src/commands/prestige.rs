use async_trait::async_trait;
use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateEmbed, EditInteractionResponse, UserId,
};
use sqlx::{Database, Pool, prelude::FromRow};
use zayden_core::FormatNum;

use crate::{Commands, Result};

const MINERS: i64 = 938_810;

#[async_trait]
pub trait PrestigeManager<Db: Database> {
    async fn row(
        pool: &Pool<Db>,
        id: impl Into<UserId> + Send,
    ) -> sqlx::Result<Option<PrestigeRow>>;
}

#[derive(FromRow, Default)]
pub struct PrestigeRow {
    pub miners: i64,
}

impl Commands {
    pub async fn prestige<Db: Database, Manager: PrestigeManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer(ctx).await?;

        let row = Manager::row(pool, interaction.user.id)
            .await
            .unwrap()
            .unwrap_or_default();

        let embed = CreateEmbed::new().description(format!(
            "❌ You need at least {} miners before you can prestige.\nYou only have {}",
            MINERS.format(),
            row.miners.format()
        ));

        interaction
            .edit_response(ctx, EditInteractionResponse::new().embed(embed))
            .await
            .unwrap();

        Ok(())
    }

    pub fn register_prestige() -> CreateCommand {
        CreateCommand::new("prestige")
            .description("Prestige your mine or casino to get unique rewards!")
    }
}
