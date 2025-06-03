use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    ResolvedOption, ResolvedValue,
};
use sqlx::{Database, Pool};

pub mod buy;
pub mod list;
pub mod sell;

pub use buy::{BuyManager, buy};
pub use list::{ListManager, list};
pub use sell::{SellManager, sell};

use crate::{GoalsManager, Result, SHOP_ITEMS, ShopPage};

use super::Commands;

impl Commands {
    pub async fn shop<
        Db: Database,
        GoalsHandler: GoalsManager<Db>,
        BuyHandler: BuyManager<Db>,
        ListHandler: ListManager<Db>,
        SaleHandler: SellManager<Db>,
    >(
        ctx: &Context,
        interaction: &CommandInteraction,
        mut options: Vec<ResolvedOption<'_>>,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer(ctx).await?;

        let command = options.pop().unwrap();

        let ResolvedValue::SubCommand(options) = command.value else {
            unreachable!("Subcommand is required")
        };

        match command.name {
            "list" => list::<Db, ListHandler>(ctx, interaction, pool).await?,
            "buy" => buy::<Db, GoalsHandler, BuyHandler>(ctx, interaction, pool, options).await?,
            "sell" => sell::<Db, SaleHandler>(ctx, interaction, pool, options).await?,
            _ => unreachable!("Invalid subcommand name"),
        };

        Ok(())
    }

    pub fn register_shop() -> CreateCommand {
        let mut page_opt = CreateCommandOption::new(CommandOptionType::String, "page", "test");

        for page in ShopPage::pages() {
            page_opt = page_opt.add_string_choice(page.to_string(), page.to_string());
        }

        let list = CreateCommandOption::new(CommandOptionType::SubCommand, "list", "Show the shop")
            .add_sub_option(page_opt);

        let mut buy_item =
            CreateCommandOption::new(CommandOptionType::String, "item", "The item to buy")
                .required(true);
        let mut sell_item =
            CreateCommandOption::new(CommandOptionType::String, "item", "The item to sell")
                .required(true);

        for si in SHOP_ITEMS.iter() {
            if si.sellable {
                sell_item = sell_item.add_string_choice(si.name, si.id);
            }

            buy_item = buy_item.add_string_choice(si.name, si.id);
        }

        let buy = CreateCommandOption::new(CommandOptionType::SubCommand, "buy", "Buy an item")
            .add_sub_option(buy_item)
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::String, "amount", "The amount to buy")
                    .required(true),
            );

        let sell = CreateCommandOption::new(CommandOptionType::SubCommand, "sell", "Sell an item")
            .add_sub_option(sell_item)
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "amount",
                    "The amount to sell",
                )
                .required(true),
            );

        CreateCommand::new("shop")
            .description("Shop commands")
            .add_option(list)
            .add_option(buy)
            .add_option(sell)
    }
}
