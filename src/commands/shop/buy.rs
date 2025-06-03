use serenity::all::{
    CommandInteraction, Context, EditInteractionResponse, ResolvedOption, ResolvedValue, UserId,
};
use sqlx::{Database, Pool, prelude::FromRow, types::Json};
use zayden_core::parse_options;

use crate::{
    Coins, Error, Gems, GoalsManager, ItemInventory, MaxBet, Result, SHOP_ITEMS, SUPER_USER,
    ShopCurrency, ShopItem, ShopPage,
    commands::shop::ShopManager,
    events::{Dispatch, Event, ShopPurchaseEvent},
    models::{GamblingItem, Mining},
};

#[derive(FromRow)]
pub struct BuyRow {
    pub id: i64,
    pub coins: i64,
    pub gems: i64,
    pub level: i32,
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
    pub tech: i64,
    pub utility: i64,
    pub production: i64,
}

impl BuyRow {
    fn new(id: impl Into<UserId> + Send) -> Self {
        let id = id.into();

        Self {
            id: id.get() as i64,
            coins: 0,
            gems: 0,
            level: 0,
            inventory: Some(Json(Vec::new())),
            miners: 0,
            mines: 0,
            land: 0,
            countries: 0,
            continents: 0,
            planets: 0,
            solar_systems: 0,
            galaxies: 0,
            universes: 0,
            prestige: 0,
            tech: 0,
            utility: 0,
            production: 0,
        }
    }
}

impl Coins for BuyRow {
    fn coins(&self) -> i64 {
        self.coins
    }

    fn coins_mut(&mut self) -> &mut i64 {
        &mut self.coins
    }
}

impl Gems for BuyRow {
    fn gems(&self) -> i64 {
        self.gems
    }

    fn gems_mut(&mut self) -> &mut i64 {
        &mut self.gems
    }
}

impl ItemInventory for BuyRow {
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

impl Mining for BuyRow {
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
        self.tech
    }

    fn utility(&self) -> i64 {
        self.utility
    }

    fn production(&self) -> i64 {
        self.production
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

impl MaxBet for BuyRow {
    fn level(&self) -> i32 {
        self.level
    }
}

pub async fn buy<Db: Database, GoalsHandler: GoalsManager<Db>, BuyHandler: ShopManager<Db>>(
    ctx: &Context,
    interaction: &CommandInteraction,
    pool: &Pool<Db>,
    options: Vec<ResolvedOption<'_>>,
) -> Result<()> {
    let mut options = parse_options(options);

    let Some(ResolvedValue::String(item)) = options.remove("item") else {
        unreachable!("item is required");
    };

    let item = SHOP_ITEMS
        .get(item)
        .expect("Preset choices so item should always exist");

    let Some(ResolvedValue::String(amount)) = options.remove("amount") else {
        unreachable!("amount is required")
    };

    let mut row = match BuyHandler::buy_row(pool, interaction.user.id).await? {
        Some(row) => row,
        None => BuyRow::new(interaction.user.id),
    };

    let costs = item
        .cost
        .iter()
        .filter_map(|x| x.as_ref())
        .copied()
        .collect::<Vec<_>>();

    let amount: i64 = match amount.parse() {
        Ok(x) => x,
        Err(_) if amount == "a" => {
            if SUPER_USER != interaction.user.id {
                return Err(Error::PremiumRequired);
            }

            match costs.first().copied() {
                Some((coins, ShopCurrency::Coins)) => row.coins() / coins,
                Some((gems, ShopCurrency::Gems)) => row.gems() / gems,
                Some(_) => unimplemented!("Currency not implimented"),
                None => unreachable!("No cost found"),
            }
        }
        _ => return Err(Error::InvalidAmount),
    };

    if amount.is_negative() {
        return Err(Error::NegativeAmount);
    }

    if amount == 0 {
        return Err(Error::ZeroAmount);
    }

    let costs = costs
        .into_iter()
        .map(|(cost, currency)| (cost * amount, currency))
        .collect::<Vec<_>>();

    for (cost, currency) in costs.iter().copied() {
        let funds = match currency {
            ShopCurrency::Coins => row.coins_mut(),
            ShopCurrency::Gems => row.gems_mut(),
            ShopCurrency::Tech => &mut row.tech,
            ShopCurrency::Utility => &mut row.utility,
            ShopCurrency::Production => &mut row.production,
            _ => unimplemented!("Currnecy not implemented"),
        };
        *funds -= cost;

        if *funds < 0 {
            return Err(Error::InsufficientFunds {
                required: funds.abs(),
                currency,
            });
        }
    }

    let quantity = if matches!(item.category, ShopPage::Mine1 | ShopPage::Mine2) {
        edit_mine(&mut row, item, amount)?
    } else {
        edit_inv(&mut row, item, amount)
    };

    Dispatch::<Db, GoalsHandler>::new(pool)
        .fire(
            &mut row,
            Event::ShopPurchase(ShopPurchaseEvent::new(interaction.user.id, item.id)),
        )
        .await?;

    BuyHandler::buy_save(pool, row).await.unwrap();

    let cost = costs
        .into_iter()
        .map(|(cost, currency)| format!("`{}` {}", cost, currency))
        .collect::<Vec<_>>();

    interaction
        .edit_response(
            ctx,
            EditInteractionResponse::new().content(format!(
                "You bought {amount} {item} for {}\nYou now have {quantity}.",
                cost.join("\n")
            )),
        )
        .await?;

    Ok(())
}

fn edit_inv(row: &mut BuyRow, item: &ShopItem<'_>, amount: i64) -> i64 {
    let inventory = row.inventory_mut();

    match inventory
        .iter_mut()
        .find(|inv_item| inv_item.item_id == item.id)
    {
        Some(item) => {
            item.quantity += amount;
            item.quantity
        }
        None => {
            let mut item = GamblingItem::from(item);
            item.quantity = amount;
            inventory.push(item);
            amount
        }
    }
}

fn edit_mine(row: &mut BuyRow, item: &ShopItem<'_>, amount: i64) -> Result<i64> {
    let value = match item.id {
        "miner" => &mut row.miners,
        "mine" => &mut row.mines,
        "land" => &mut row.land,
        "country" => &mut row.countries,
        "continent" => &mut row.continents,
        "planet" => &mut row.planets,
        "solar_system" => &mut row.solar_systems,
        "galaxy" => &mut row.galaxies,
        "universe" => &mut row.universes,
        _ => unreachable!("Invalid item id {}", item.id),
    };

    *value += amount;

    let quantity = *value;

    if quantity > *row.max_values().get(item.id).unwrap() {
        return Err(Error::InvalidAmount);
    }

    Ok(quantity)
}
