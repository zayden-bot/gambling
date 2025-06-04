use std::fmt::Display;

use async_trait::async_trait;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbed, EditInteractionResponse, Mentionable, ResolvedOption, ResolvedValue, UserId,
};
use sqlx::types::Json;
use sqlx::{Database, Pool, prelude::FromRow};
use zayden_core::parse_options;

use crate::shop::{LUCKY_CHIP, SHOP_ITEMS, ShopCurrency, ShopItem, ShopPage};
use crate::{
    COIN, Coins, EffectsManager, Error, GEM, GamblingItem, Gems, ItemInventory, Mining, Result,
};

use super::Commands;

struct InventoryItem<'a> {
    id: &'a str,
    name: &'a str,
    emoji: String,
    cost: [Option<(i64, ShopCurrency)>; 4],
    quantity: i64,
}

impl Display for InventoryItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.emoji, self.name)
    }
}

impl<'a> From<&ShopItem<'a>> for InventoryItem<'a> {
    fn from(value: &ShopItem<'a>) -> Self {
        Self {
            id: value.id,
            name: value.name,
            emoji: value.emoji(),
            cost: value.cost,
            quantity: 0,
        }
    }
}

#[async_trait]
pub trait InventoryManager<Db: Database> {
    async fn row(
        pool: &Pool<Db>,
        id: impl Into<UserId> + Send,
    ) -> sqlx::Result<Option<InventoryRow>>;

    async fn edit_item_quantity(
        pool: &Pool<Db>,
        id: impl Into<UserId> + Send,
        item_id: &str,
        amount: i64,
    ) -> sqlx::Result<i64>;
}

#[derive(Default, FromRow)]
pub struct InventoryRow {
    pub coins: i64,
    pub gems: i64,
    pub inventory: Option<Json<Vec<GamblingItem>>>,
    pub tech: i64,
    pub utility: i64,
    pub production: i64,
    pub coal: i64,
    pub iron: i64,
    pub gold: i64,
    pub redstone: i64,
    pub lapis: i64,
    pub diamonds: i64,
    pub emeralds: i64,
}

impl Coins for InventoryRow {
    fn coins(&self) -> i64 {
        self.coins
    }

    fn coins_mut(&mut self) -> &mut i64 {
        &mut self.coins
    }
}

impl Gems for InventoryRow {
    fn gems(&self) -> i64 {
        self.gems
    }

    fn gems_mut(&mut self) -> &mut i64 {
        &mut self.gems
    }
}

impl ItemInventory for InventoryRow {
    fn inventory(&self) -> &[GamblingItem] {
        match self.inventory.as_ref() {
            Some(vec_ref) => &vec_ref.0,
            None => &[],
        }
    }

    fn inventory_mut(&mut self) -> &mut Vec<GamblingItem> {
        self.inventory.get_or_insert_with(|| Json(Vec::new()))
    }
}

impl Mining for InventoryRow {
    fn miners(&self) -> i64 {
        unimplemented!()
    }

    fn mines(&self) -> i64 {
        unimplemented!()
    }

    fn land(&self) -> i64 {
        unimplemented!()
    }

    fn countries(&self) -> i64 {
        unimplemented!()
    }

    fn continents(&self) -> i64 {
        unimplemented!()
    }

    fn planets(&self) -> i64 {
        unimplemented!()
    }

    fn solar_systems(&self) -> i64 {
        unimplemented!()
    }

    fn galaxies(&self) -> i64 {
        unimplemented!()
    }

    fn universes(&self) -> i64 {
        unimplemented!()
    }

    fn prestige(&self) -> i64 {
        unimplemented!()
    }

    fn tech(&self) -> i64 {
        self.tech
    }

    fn utility(&self) -> i64 {
        self.utility
    }

    fn production(&self) -> i64 {
        self.production
    }

    fn coal(&self) -> i64 {
        self.coal
    }

    fn iron(&self) -> i64 {
        self.iron
    }

    fn gold(&self) -> i64 {
        self.gold
    }

    fn redstone(&self) -> i64 {
        self.redstone
    }

    fn lapis(&self) -> i64 {
        self.lapis
    }

    fn diamonds(&self) -> i64 {
        self.diamonds
    }

    fn emeralds(&self) -> i64 {
        self.emeralds
    }
}

impl Commands {
    pub async fn inventory<
        Db: Database,
        EffectsHandler: EffectsManager<Db>,
        InventoryHandler: InventoryManager<Db>,
    >(
        ctx: &Context,
        interaction: &CommandInteraction,
        mut options: Vec<ResolvedOption<'_>>,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer(ctx).await.unwrap();

        let subcommand = options.pop().unwrap();

        match subcommand.name {
            "show" => show::<Db, InventoryHandler>(ctx, interaction, pool).await,
            "use" => {
                let ResolvedValue::SubCommand(options) = subcommand.value else {
                    unreachable!("Option must be a subcommand")
                };

                use_item::<Db, EffectsHandler, InventoryHandler>(ctx, interaction, options, pool)
                    .await
            }
            _ => unreachable!("Invalid subcommand"),
        }
    }

    pub fn register_inventory() -> CreateCommand {
        let item_opt = CreateCommandOption::new(
            CommandOptionType::String,
            "item",
            "Select the item you want to activate",
        )
        .required(true)
        .add_string_choice(LUCKY_CHIP.name, LUCKY_CHIP.id);

        // for item in SHOP_ITEMS.iter().filter(|item| item.useable) {
        //     item_opt = item_opt.add_string_choice(item.name, item.id)
        // }

        let use_item = CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "use",
            "Activate an item in your inventory",
        )
        .add_sub_option(item_opt)
        .add_sub_option(CreateCommandOption::new(
            CommandOptionType::String,
            "amount",
            "Enter the number of items to activate",
        ));

        CreateCommand::new("inventory")
            .description("Inventory commands")
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "show",
                "Show your inventory and any active items",
            ))
            .add_option(use_item)
    }
}

async fn show<Db: Database, Manager: InventoryManager<Db>>(
    ctx: &Context,
    interaction: &CommandInteraction,
    pool: &Pool<Db>,
) -> Result<()> {
    let row = Manager::row(pool, interaction.user.id)
        .await
        .unwrap()
        .unwrap_or_default();

    let (items, boosts) = SHOP_ITEMS
        .iter()
        .filter(|item| {
            matches!(
                item.category,
                ShopPage::Item | ShopPage::Boost1 | ShopPage::Boost2
            )
        })
        .map(InventoryItem::from)
        .map(|mut item| {
            if let Some(inv_item) = row
                .inventory()
                .iter()
                .find(|inv_item| inv_item.item_id == item.id)
            {
                item.quantity = inv_item.quantity;
            }

            item
        })
        .partition::<Vec<_>, _>(|item| matches!(item.cost[0], Some((_, ShopCurrency::Coins))));

    let mut embed = CreateEmbed::new()
        .field(
            "Currencies",
            format!(
                "<:coin:{COIN}> {} coins\n{GEM} {} gems",
                row.coins_str(),
                row.gems_str()
            ),
            false,
        )
        .field(
            "Items",
            items
                .into_iter()
                .map(|item| format!("{} `{}` {}", item.emoji, item.quantity, item.name))
                .collect::<Vec<_>>()
                .join("\n"),
            true,
        )
        .field(
            "Boosts",
            boosts
                .into_iter()
                .map(|item| format!("{} `{}` {}", item.emoji, item.quantity, item.name))
                .collect::<Vec<_>>()
                .join("\n"),
            true,
        )
        .field("Resources", row.resources(), true)
        .field("Crafted", row.crafted(), false)
        .field(
            "Weapons",
            format!(
                "{} is fighting with just their fists 👊",
                interaction.user.mention()
            ),
            false,
        );

    if let Some(avatar) = interaction.user.avatar_url() {
        embed = embed.thumbnail(avatar);
    }

    interaction
        .edit_response(ctx, EditInteractionResponse::new().embed(embed))
        .await
        .unwrap();

    Ok(())
}

async fn use_item<
    Db: Database,
    EffectsHandler: EffectsManager<Db>,
    InventoryHandler: InventoryManager<Db>,
>(
    ctx: &Context,
    interaction: &CommandInteraction,
    options: Vec<ResolvedOption<'_>>,
    pool: &Pool<Db>,
) -> Result<()> {
    let mut options = parse_options(options);

    let Some(ResolvedValue::String(item_id)) = options.remove("item") else {
        unreachable!("item is required option")
    };

    let item = SHOP_ITEMS.get(item_id).unwrap();

    let amount = match options.remove("amount") {
        Some(ResolvedValue::String(amount)) => amount.parse().map_err(|_| Error::InvalidAmount)?,
        _ => 1,
    };

    if amount < 0 {
        return Err(Error::NegativeAmount);
    }

    if amount == 0 {
        return Err(Error::ZeroAmount);
    }

    let quantity = match InventoryHandler::edit_item_quantity(
        pool,
        interaction.user.id,
        item_id,
        amount,
    )
    .await
    {
        Ok(q) => q,
        Err(sqlx::Error::RowNotFound) => return Err(Error::InvalidAmount),
        r => r?,
    };

    let mut tx = pool.begin().await.unwrap();

    for _ in 0..amount {
        EffectsHandler::add_effect(&mut *tx, interaction.user.id, item)
            .await
            .unwrap();
    }

    tx.commit().await.unwrap();

    let embed = CreateEmbed::new().description(format!(
        "Successfully activated item:\n**{item}**\nUses left:{}",
        quantity
    ));

    interaction
        .edit_response(ctx, EditInteractionResponse::new().embed(embed))
        .await
        .unwrap();

    Ok(())
}
