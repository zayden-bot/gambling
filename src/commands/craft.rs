use async_trait::async_trait;
use serenity::all::{
    Colour, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbed, EditInteractionResponse, ResolvedOption, ResolvedValue, UserId,
};
use sqlx::any::AnyQueryResult;
use sqlx::prelude::FromRow;
use sqlx::{Database, Pool};
use zayden_core::parse_options;

use crate::shop::ShopCurrency;
use crate::{Error, Result};

use super::Commands;

#[async_trait]
pub trait CraftManager<Db: Database> {
    async fn row(pool: &Pool<Db>, id: impl Into<UserId> + Send) -> sqlx::Result<CraftRow>;

    async fn save(pool: &Pool<Db>, row: CraftRow) -> sqlx::Result<AnyQueryResult>;
}

#[derive(FromRow)]
pub struct CraftRow {
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

impl Commands {
    pub async fn craft<Db: Database, Manager: CraftManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        options: Vec<ResolvedOption<'_>>,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer(ctx).await.unwrap();

        let mut row = Manager::row(pool, interaction.user.id).await.unwrap();

        let mut options = parse_options(options);

        if !options.contains_key("type") {
            menu(ctx, interaction, row).await;
            return Ok(());
        }

        let Some(ResolvedValue::String(type_)) = options.remove("type") else {
            unreachable!("Type must be present")
        };

        let amount = match options.remove("amount") {
            Some(ResolvedValue::Integer(amount)) => amount,
            _ => 1,
        };

        if amount.is_negative() {
            return Err(Error::NegativeAmount);
        }

        if amount == 0 {
            return Err(Error::ZeroAmount);
        }

        let item: ShopCurrency = type_.parse().unwrap();

        let costs = item
            .craft_req()
            .into_iter()
            .flatten()
            .map(|(currency, cost)| (currency, cost as i64 * amount))
            .collect::<Vec<_>>();

        for (currency, cost) in costs {
            let fund = match currency {
                ShopCurrency::Coal => &mut row.coal,
                ShopCurrency::Iron => &mut row.iron,
                ShopCurrency::Gold => &mut row.gold,
                ShopCurrency::Redstone => &mut row.redstone,
                ShopCurrency::Lapis => &mut row.lapis,
                ShopCurrency::Diamonds => &mut row.diamonds,
                ShopCurrency::Emeralds => &mut row.emeralds,
                c => unreachable!("Invalid crafting currency: {c}"),
            };

            *fund -= cost;
            if *fund < 0 {
                return Err(Error::InsufficientFunds {
                    required: fund.abs(),
                    currency,
                });
            }
        }

        let quantity = match item {
            ShopCurrency::Tech => {
                row.tech += amount;
                row.tech
            }
            ShopCurrency::Utility => {
                row.utility += amount;
                row.utility
            }
            ShopCurrency::Production => {
                row.production += amount;
                row.production
            }
            c => unreachable!("Invalid item: {c}"),
        };

        Manager::save(pool, row).await.unwrap();

        let embed = CreateEmbed::new()
            .description(format!(
                "Crafted `{amount}` {item:?}s\nYou now  have `{quantity}` {item:?}s"
            ))
            .colour(Colour::ORANGE);

        interaction
            .edit_response(ctx, EditInteractionResponse::new().embed(embed))
            .await
            .unwrap();

        Ok(())
    }

    pub fn register_craft() -> CreateCommand {
        CreateCommand::new("craft")
            .description("Craft packs to buy mining units")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "type",
                    "The type of pack to craft",
                )
                .add_string_choice("Tech Pack", "tech")
                .add_string_choice("Utility Pack", "utility")
                .add_string_choice("Production Pack", "production"),
            )
            .add_option(CreateCommandOption::new(
                CommandOptionType::Integer,
                "amount",
                "The amount to craft",
            ))
    }
}

async fn menu(ctx: &Context, interaction: &CommandInteraction, row: CraftRow) {
    let mut desc = [
        ShopCurrency::Coal,
        ShopCurrency::Iron,
        ShopCurrency::Gold,
        ShopCurrency::Redstone,
        ShopCurrency::Lapis,
        ShopCurrency::Diamonds,
        ShopCurrency::Emeralds,
    ]
    .into_iter()
    .map(|item| match item {
        ShopCurrency::Coal => (item, row.coal),
        ShopCurrency::Iron => (item, row.iron),
        ShopCurrency::Gold => (item, row.gold),
        ShopCurrency::Redstone => (item, row.redstone),
        ShopCurrency::Lapis => (item, row.lapis),
        ShopCurrency::Diamonds => (item, row.diamonds),
        ShopCurrency::Emeralds => (item, row.emeralds),
        _ => unreachable!(),
    })
    .map(|(item, owned)| format!("`{owned}` {item}"))
    .collect::<Vec<_>>()
    .join("\n");
    desc.push_str("\n------------------\n");

    desc.push_str(
        &[
            ShopCurrency::Tech,
            ShopCurrency::Utility,
            ShopCurrency::Production,
        ]
        .into_iter()
        .map(|item| match item {
            ShopCurrency::Tech => (item, row.tech),
            ShopCurrency::Utility => (item, row.utility),
            ShopCurrency::Production => (item, row.production),
            _ => unreachable!(),
        })
        .map(|(item, owned)| {
            format!(
                "**{item:?}**\nOwned: `{owned}`\n{}",
                item.craft_req()
                    .into_iter()
                    .flatten()
                    .map(|(currency, cost)| format!("`{cost}` {currency}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n"),
    );

    desc.push_str("\n------------------\n`/craft <id> <amount>`");

    let embed = CreateEmbed::new()
        .title("Craftable Items")
        .description(desc)
        .colour(Colour::ORANGE);

    interaction
        .edit_response(ctx, EditInteractionResponse::new().embed(embed))
        .await
        .unwrap();
}
