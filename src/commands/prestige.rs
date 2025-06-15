use std::time::Duration;

use async_trait::async_trait;
use futures::StreamExt;
use serenity::all::{
    ButtonStyle, Colour, CommandInteraction, Context, CreateButton, CreateCommand, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, EditInteractionResponse, UserId,
};
use sqlx::{Database, Pool, any::AnyQueryResult, prelude::FromRow};
use zayden_core::FormatNum;

use crate::{Commands, Result};

const MINERS: i64 = 938_810;

#[async_trait]
pub trait PrestigeManager<Db: Database> {
    async fn row(
        pool: &Pool<Db>,
        id: impl Into<UserId> + Send,
    ) -> sqlx::Result<Option<PrestigeRow>>;

    async fn save(pool: &Pool<Db>, row: PrestigeRow) -> sqlx::Result<AnyQueryResult>;
}

#[derive(FromRow, Default)]
pub struct PrestigeRow {
    pub id: i64,
    pub miners: i64,
    pub prestige: i64,
}

impl Commands {
    pub async fn prestige<Db: Database, Manager: PrestigeManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer(ctx).await?;

        let mut row = Manager::row(pool, interaction.user.id)
            .await
            .unwrap()
            .unwrap_or_default();

        if row.miners < MINERS {
            let embed = CreateEmbed::new()
                .description(format!(
                    "❌ You need at least `{}` miners before you can prestige.\nYou only have `{}`",
                    MINERS.format(),
                    row.miners.format()
                ))
                .colour(Colour::RED);

            interaction
                .edit_response(ctx, EditInteractionResponse::new().embed(embed))
                .await
                .unwrap();

            return Ok(());
        }
        // Are you sure you want to **prestige** your mine?\n\nPrestiging will **reset your current mine progress**, but you'll unlock powerful upgrades!
        let embed = CreateEmbed::new().description("Are you sure you want to **prestige** your mine?\n\nPrestiging will **reset your current mine progress**, but you'll unlock powerful upgrades!").colour(Colour::TEAL);

        let confirm = CreateButton::new("confirm")
            .label("Confirm")
            .emoji('✅')
            .style(ButtonStyle::Primary);
        let cancel = CreateButton::new("cancel")
            .label("Cancel")
            .emoji('❌')
            .style(ButtonStyle::Secondary);

        let msg = interaction
            .edit_response(
                ctx,
                EditInteractionResponse::new()
                    .embed(embed)
                    .button(confirm)
                    .button(cancel),
            )
            .await
            .unwrap();

        let mut stream = msg
            .await_component_interactions(ctx)
            .timeout(Duration::from_secs(120))
            .stream();

        if let Some(component) = stream.next().await {
            if component.data.custom_id == "confirm" {
                row.prestige += 1;
                Manager::save(pool, row).await.unwrap();

                component
                    .create_response(
                        ctx,
                        CreateInteractionResponse::UpdateMessage(
                            CreateInteractionResponseMessage::new()
                                .content("Done")
                                .components(Vec::new()),
                        ),
                    )
                    .await
                    .unwrap();

                return Ok(());
            }

            component
                .create_response(ctx, CreateInteractionResponse::Acknowledge)
                .await
                .unwrap();
        }

        msg.delete(ctx).await.unwrap();

        Ok(())
    }

    pub fn register_prestige() -> CreateCommand {
        CreateCommand::new("prestige")
            .description("Prestige your mine or casino to get unique rewards!")
    }
}
