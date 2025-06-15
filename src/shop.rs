use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::str::FromStr;
use std::time::Duration;

use crate::{COAL, COIN, DIAMOND, EMERALD, GOLD, GamblingItem, IRON, LAPIS, REDSTONE};

pub const SALES_TAX: f64 = 0.1;

#[derive(Clone, Copy)]
pub enum ShopCurrency {
    Coins,
    Gems,
    Tech,
    Utility,
    Production,
    Coal,
    Iron,
    Gold,
    Redstone,
    Lapis,
    Diamonds,
    Emeralds,
}

impl ShopCurrency {
    pub fn craft_req(&self) -> [Option<(Self, u16)>; 4] {
        match self {
            Self::Tech => [Some((Self::Coal, 10)), Some((Self::Iron, 5)), None, None],
            Self::Utility => [
                Some((Self::Coal, 15)),
                Some((Self::Gold, 10)),
                Some((Self::Diamonds, 5)),
                Some((Self::Emeralds, 1)),
            ],
            Self::Production => [
                Some((Self::Gold, 100)),
                Some((Self::Lapis, 500)),
                Some((Self::Redstone, 125)),
                None,
            ],
            c => unreachable!("Invalid currency {c}"),
        }
    }
}

impl Debug for ShopCurrency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Coins => write!(f, "Coins"),
            Self::Gems => write!(f, "Gems"),
            Self::Tech => write!(f, "Tech Pack"),
            Self::Utility => write!(f, "Utility Pack"),
            Self::Production => write!(f, "Production Pack"),
            Self::Coal => write!(f, "Coal"),
            Self::Iron => write!(f, "Iron"),
            Self::Gold => write!(f, "Gold"),
            Self::Redstone => write!(f, "Redstone"),
            Self::Lapis => write!(f, "Lapis"),
            Self::Diamonds => write!(f, "Diamonds"),
            Self::Emeralds => write!(f, "Emeralds"),
        }
    }
}

impl Display for ShopCurrency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Coins => write!(f, "<:coin:{COIN}>"),
            Self::Gems => write!(f, "💎"),
            Self::Tech => write!(f, "Tech"),
            Self::Utility => write!(f, "Utility"),
            Self::Production => write!(f, "Production"),
            Self::Coal => write!(f, "<:coal:{COAL}>"),
            Self::Iron => write!(f, "<:iron:{IRON}>"),
            Self::Gold => write!(f, "<:gold:{GOLD}>"),
            Self::Redstone => write!(f, "<:redstone:{REDSTONE}>"),
            Self::Lapis => write!(f, "<:lapis:{LAPIS}>"),
            Self::Diamonds => write!(f, "<:diamond:{DIAMOND}>"),
            Self::Emeralds => write!(f, "<:emerald:{EMERALD}>"),
        }
    }
}

impl FromStr for ShopCurrency {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "tech" => Ok(Self::Tech),
            "utility" => Ok(Self::Utility),
            "production" => Ok(Self::Production),
            s => unimplemented!("Currency {s} has not been implemented"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShopPage {
    Item,
    Boost1,
    Boost2,
    Mine1,
    Mine2,
}

impl ShopPage {
    pub const fn pages() -> [ShopPage; 5] {
        [
            ShopPage::Item,
            ShopPage::Boost1,
            ShopPage::Boost2,
            ShopPage::Mine1,
            ShopPage::Mine2,
        ]
    }
}

impl Display for ShopPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Item => write!(f, "Item"),
            Self::Boost1 => write!(f, "Boost 1"),
            Self::Boost2 => write!(f, "Boost 2"),
            Self::Mine1 => write!(f, "Mine 1"),
            Self::Mine2 => write!(f, "Mine 2"),
        }
    }
}

impl FromStr for ShopPage {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Item" => Ok(Self::Item),
            "Boost 1" => Ok(Self::Boost1),
            "Boost 2" => Ok(Self::Boost2),
            "Mine 1" => Ok(Self::Mine1),
            "Mine 2" => Ok(Self::Mine2),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ShopItem<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub emoji: &'a str,
    pub description: &'a str,
    pub cost: [Option<(i64, ShopCurrency)>; 4],
    pub category: ShopPage,
    pub sellable: bool,
    pub useable: bool,
    pub effect_fn: fn(i64, i64) -> i64,
    pub effect_duration: Option<Duration>,
}

impl<'a> ShopItem<'a> {
    const fn new(
        id: &'a str,
        name: &'a str,
        emoji: &'a str,
        desc: &'a str,
        cost: i64,
        currency: ShopCurrency,
        category: ShopPage,
    ) -> ShopItem<'a> {
        ShopItem {
            id,
            name,
            emoji,
            description: desc,
            cost: [Some((cost, currency)), None, None, None],
            category,
            sellable: false,
            useable: false,
            effect_fn: |_, payout| payout,
            effect_duration: None,
        }
    }

    const fn add_cost(mut self, cost: i64, currency: ShopCurrency) -> ShopItem<'a> {
        let mut i = 0;
        while i < self.cost.len() {
            if self.cost[i].is_none() {
                self.cost[i] = Some((cost, currency));
                break;
            }

            i += 1;
        }

        self
    }

    const fn sellable(mut self, value: bool) -> ShopItem<'a> {
        self.sellable = value;
        self
    }

    const fn useable(mut self, value: bool) -> ShopItem<'a> {
        self.useable = value;
        self
    }

    const fn effect_fn(mut self, f: fn(i64, i64) -> i64) -> Self {
        self.effect_fn = f;
        self
    }

    const fn duration(mut self, d: Duration) -> Self {
        self.effect_duration = Some(d);
        self
    }

    pub fn emoji(&self) -> String {
        match self.emoji.parse::<i64>() {
            Ok(id) => format!("<:{}:{id}>", self.id),
            Err(_) => String::from(self.emoji),
        }
    }

    pub fn cost_desc(&self) -> String {
        self.cost
            .iter()
            .filter_map(|cost| cost.as_ref())
            .map(|(cost, currency)| format!("`{cost}` {currency}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn coin_cost(&self) -> Option<i64> {
        self.cost
            .iter()
            .filter_map(|x| x.as_ref())
            .find(|(_, currency)| matches!(currency, ShopCurrency::Coins))
            .map(|(cost, _)| cost)
            .copied()
    }
}

impl Display for ShopItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.emoji(), self.name)
    }
}

impl From<&GamblingItem> for ShopItem<'_> {
    fn from(value: &GamblingItem) -> Self {
        *SHOP_ITEMS
            .iter()
            .find(|item| item.id == value.item_id)
            .unwrap()
    }
}

pub const LOTTO_TICKET: ShopItem = ShopItem::new(
    "lottoticket",
    "Lottery Ticket",
    "🎟️",
    "Enter the daily lottery.\nThe more tickets bought have the higher the jackpot.",
    5_000,
    ShopCurrency::Coins,
    ShopPage::Item,
);

pub const EGGPLANT: ShopItem = ShopItem::new(
    "eggplant",
    "Eggplant",
    "🍆",
    "Who has the biggest eggplant?",
    10_000,
    ShopCurrency::Coins,
    ShopPage::Item,
)
.sellable(true);

pub const WEAPON_CRATE: ShopItem = ShopItem::new(
    "weaponcrate",
    "Weapon Crate",
    "📦",
    "Unlock for a weapon to display on your profile",
    100_000,
    ShopCurrency::Coins,
    ShopPage::Item,
)
.sellable(true)
.useable(true);

pub const LUCKY_CHIP: ShopItem = ShopItem::new(
    "luckychip",
    "Lucky Chip",
    "⭐",
    "Refund your bet if you lose",
    3,
    ShopCurrency::Gems,
    ShopPage::Boost1,
)
.useable(true)
.effect_fn(|bet, _| bet);

const RIGGED_LUCK: ShopItem = ShopItem::new(
    "riggedluck",
    "Rigged Luck",
    "⚪",
    "Double your chances! Your win probability is increased by 100% for the next game. (Max 75% total win chance)",
    30,
    ShopCurrency::Gems,
    ShopPage::Boost1,
).useable(true);

const PAYOUT_X2: ShopItem = ShopItem::new(
    "payout2x",
    "Profit x2",
    "⚪",
    "Double payout from winning | Duration: `+15 minute`",
    2,
    ShopCurrency::Gems,
    ShopPage::Boost2,
)
.useable(true)
.effect_fn(|_, payout| {
    if payout < 0 {
        return payout;
    }

    payout * 2
})
.duration(Duration::from_secs(15 * 60));

const PAYOUT_X5: ShopItem = ShopItem::new(
    "payout5x",
    "Profit x5",
    "⚪",
    "Five times payout from winning | Duration: `+10 minute`",
    5,
    ShopCurrency::Gems,
    ShopPage::Boost2,
)
.useable(true)
.effect_fn(|_, payout| {
    if payout < 0 {
        return payout;
    }

    payout * 5
})
.duration(Duration::from_secs(10 * 60));

const PAYOUT_X10: ShopItem = ShopItem::new(
    "payout10x",
    "Profit x10",
    "⚪",
    "Ten times payout from winning | Duration: `+5 minute`",
    10,
    ShopCurrency::Gems,
    ShopPage::Boost2,
)
.useable(true)
.effect_fn(|_, payout| {
    if payout < 0 {
        return payout;
    }

    payout * 10
})
.duration(Duration::from_secs(5 * 60));

const PAYOUT_X50: ShopItem = ShopItem::new(
    "payout50x",
    "Profit x50",
    "⚪",
    "Fifty times payout from winning | Duration: `+2 minute`",
    25,
    ShopCurrency::Gems,
    ShopPage::Boost2,
)
.useable(true)
.effect_fn(|_, payout| {
    if payout < 0 {
        return payout;
    }

    payout * 50
})
.duration(Duration::from_secs(2 * 60));

const PAYOUT_X100: ShopItem = ShopItem::new(
    "payout100x",
    "Profit x100",
    "⚪",
    "One hundered times payout from winning | Duration: `+1 minute`",
    50,
    ShopCurrency::Gems,
    ShopPage::Boost2,
)
.useable(true)
.effect_fn(|_, payout| {
    if payout < 0 {
        return payout;
    }

    payout * 100
})
.duration(Duration::from_secs(60));

//region: Mine
const MINER_COST: i64 = 100;
const MINE_COST: i64 = MINER_COST * 5;
const LAND_COST: i64 = MINE_COST * 20;
const COUNTRY_COST: i64 = LAND_COST * 10;
const CONTINENT_COST: i64 = COUNTRY_COST * 10;
const PLANET_COST: i64 = CONTINENT_COST * 5;
const SOLAR_SYSTEM_COST: i64 = PLANET_COST * 20;
const GALAXY_COST: i64 = SOLAR_SYSTEM_COST * 10;
const UNIVERSE_COST: i64 = GALAXY_COST * 10;

const MINER: ShopItem = ShopItem::new(
    "miner",
    "Miner",
    "",
    "Increases passive mine income and boosts resource gains from dig",
    MINER_COST,
    ShopCurrency::Coins,
    ShopPage::Mine1,
);

const MINE: ShopItem = ShopItem::new(
    "mine",
    "Mine",
    "",
    "Allows you to hire 10 extra miners per mine",
    MINE_COST,
    ShopCurrency::Coins,
    ShopPage::Mine1,
)
.add_cost(1, ShopCurrency::Tech);

const LAND: ShopItem = ShopItem::new(
    "land",
    "Land",
    "",
    "Allows you to buy 5 extra mines per land",
    LAND_COST,
    ShopCurrency::Coins,
    ShopPage::Mine1,
)
.add_cost(10, ShopCurrency::Tech);

const COUNTRY: ShopItem = ShopItem::new(
    "country",
    "Country",
    "",
    "Allows you to buy 25 extra plots of land per country",
    COUNTRY_COST,
    ShopCurrency::Coins,
    ShopPage::Mine1,
)
.add_cost(250, ShopCurrency::Tech)
.add_cost(10, ShopCurrency::Utility);

const CONTINENT: ShopItem = ShopItem::new(
    "continent",
    "Continent",
    "",
    "Allows you to buy 50 extra countries per continent",
    CONTINENT_COST,
    ShopCurrency::Coins,
    ShopPage::Mine1,
)
.add_cost(5000, ShopCurrency::Tech)
.add_cost(500, ShopCurrency::Utility)
.add_cost(100, ShopCurrency::Production);

const PLANET: ShopItem = ShopItem::new(
    "planet",
    "Planet",
    "",
    "Allows you to buy 7 extra continents per planet",
    PLANET_COST,
    ShopCurrency::Coins,
    ShopPage::Mine2,
)
.add_cost(10000, ShopCurrency::Tech)
.add_cost(2500, ShopCurrency::Utility)
.add_cost(1000, ShopCurrency::Production);

const SOLAR_SYSTEM: ShopItem = ShopItem::new(
    "solarsystem",
    "Solar System",
    "",
    "Allows you to buy 8 extra planets per solar system",
    SOLAR_SYSTEM_COST,
    ShopCurrency::Coins,
    ShopPage::Mine2,
)
.add_cost(25000, ShopCurrency::Tech)
.add_cost(10000, ShopCurrency::Utility)
.add_cost(5000, ShopCurrency::Production);

const GALAXY: ShopItem = ShopItem::new(
    "galaxy",
    "Galaxy",
    "",
    "Allows you to buy 100 extra planets per solar system",
    GALAXY_COST,
    ShopCurrency::Coins,
    ShopPage::Mine2,
)
.add_cost(50000, ShopCurrency::Tech)
.add_cost(25000, ShopCurrency::Utility)
.add_cost(10000, ShopCurrency::Production);

const UNIVERSE: ShopItem = ShopItem::new(
    "universe",
    "Universe",
    "",
    "Allows you to buy 255 extra galaxies per universe",
    UNIVERSE_COST,
    ShopCurrency::Coins,
    ShopPage::Mine2,
)
.add_cost(1000000, ShopCurrency::Tech)
.add_cost(1000000, ShopCurrency::Utility)
.add_cost(1000000, ShopCurrency::Production);
//endregion

pub struct ShopItems<'a>([ShopItem<'a>; 17]);

impl ShopItems<'_> {
    pub fn get(&self, id: &str) -> Option<&ShopItem> {
        self.0.iter().find(|item| item.id == id)
    }
}

impl<'a> Deref for ShopItems<'a> {
    type Target = [ShopItem<'a>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub const SHOP_ITEMS: ShopItems = ShopItems([
    LOTTO_TICKET,
    EGGPLANT,
    // WEAPON_CRATE,
    LUCKY_CHIP,
    // RIGGED_LUCK,
    PAYOUT_X2,
    PAYOUT_X5,
    PAYOUT_X10,
    PAYOUT_X50,
    PAYOUT_X100,
    MINER,
    MINE,
    LAND,
    COUNTRY,
    CONTINENT,
    PLANET,
    SOLAR_SYSTEM,
    GALAXY,
    UNIVERSE,
]);
