use std::time::Duration;

use async_trait::async_trait;
use futures::StreamExt;
use serenity::all::{
    ButtonStyle, Colour, CommandInteraction, Context, CreateButton, CreateCommand, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, EditInteractionResponse, UserId,
};
use sqlx::any::AnyQueryResult;
use sqlx::types::Json;
use sqlx::{Database, FromRow, Pool};
use zayden_core::FormatNum;

use crate::shop::LOTTO_TICKET;
use crate::{Commands, GamblingItem, Result, SHOP_ITEMS, START_AMOUNT};

const MINERS: i64 = 938_810;

#[async_trait]
pub trait PrestigeManager<Db: Database> {
    async fn miners(pool: &Pool<Db>, id: impl Into<UserId> + Send) -> sqlx::Result<Option<i64>>;

    async fn row(
        pool: &Pool<Db>,
        id: impl Into<UserId> + Send,
    ) -> sqlx::Result<Option<PrestigeRow>>;

    async fn save(pool: &Pool<Db>, row: PrestigeRow) -> sqlx::Result<AnyQueryResult>;
}

#[derive(FromRow, Default)]
pub struct PrestigeRow {
    pub id: i64,
    pub coins: i64,
    pub gems: i64,
    pub stamina: i64,
    pub inventory: Option<Json<Vec<GamblingItem>>>,
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
    pub coal: i64,
    pub iron: i64,
    pub gold: i64,
    pub redstone: i64,
    pub lapis: i64,
    pub diamonds: i64,
    pub emeralds: i64,
    pub tech: i64,
    pub utility: i64,
    pub production: i64,
}

impl PrestigeRow {
    pub fn prestige(&mut self) {
        self.coins = START_AMOUNT;
        self.gems += 1;
        self.stamina = 3;
        self.inventory
            .as_mut()
            .unwrap_or(&mut Json(Vec::new()))
            .retain(|item| {
                let is_sellable = SHOP_ITEMS
                    .get(&item.item_id)
                    .is_some_and(|shop_item_data| shop_item_data.sellable);

                item.item_id != LOTTO_TICKET.id && !is_sellable
            });
        self.miners = 0;
        self.mines = 0;
        self.land = 0;
        self.countries = 0;
        self.continents = 0;
        self.planets = 0;
        self.solar_systems = 0;
        self.galaxies = 0;
        self.universes = 0;
        self.prestige += 1;
        self.coal = 0;
        self.iron = 0;
        self.gold = 0;
        self.redstone = 0;
        self.lapis = 0;
        self.diamonds = 0;
        self.emeralds = 0;
        self.tech = 0;
        self.utility = 0;
        self.production = 0;
    }
}

impl Commands {
    pub async fn prestige<Db: Database, Manager: PrestigeManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer(ctx).await?;

        let miners = Manager::miners(pool, interaction.user.id)
            .await
            .unwrap()
            .unwrap_or_default();

        if miners < MINERS {
            let embed = CreateEmbed::new()
                .description(format!(
                    "❌ You need at least `{}` miners before you can prestige.\nYou only have `{}`",
                    MINERS.format(),
                    miners.format()
                ))
                .colour(Colour::RED);

            interaction
                .edit_response(ctx, EditInteractionResponse::new().embed(embed))
                .await
                .unwrap();

            return Ok(());
        }

        let embed = CreateEmbed::new().description("Are you sure you want to prestige your mine?\n\nPrestiging will **reset your mine, coins, items and resources**, but you'll unlock powerful upgrades!").colour(Colour::TEAL);

        let confirm = CreateButton::new("confirm")
            .label("Confirm")
            .emoji('✅')
            .style(ButtonStyle::Secondary);
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
            .author_id(interaction.user.id)
            .timeout(Duration::from_secs(120))
            .stream();

        if let Some(component) = stream.next().await {
            if component.data.custom_id == "confirm" {
                let mut row = Manager::row(pool, interaction.user.id)
                    .await
                    .unwrap()
                    .unwrap_or_else(|| todo!());

                row.prestige();

                println!("New Inv: {:?}", row.inventory);

                Manager::save(pool, row).await.unwrap();

                component
                    .create_response(
                        ctx,
                        CreateInteractionResponse::UpdateMessage(
                            CreateInteractionResponseMessage::new()
                                .content("Done (message wip)")
                                .embeds(Vec::new())
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
