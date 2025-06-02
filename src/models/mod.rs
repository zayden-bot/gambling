use std::collections::HashMap;

use chrono::{Duration, NaiveDateTime, Utc};

mod gambling_item;
// mod gambling;
mod gambling_effects;
mod gambling_goals;
// mod gambling_inventory;
// mod gambling_level;
// mod gambling_profile;
mod game_row;

// pub use gambling::{GamblingRow, GamblingTable};
pub use gambling_effects::{EffectsManager, EffectsRow};
// pub use gambling_goals::{GamblingGoalsRow, GamblingGoalsTable};
// pub use gambling_inventory::{GamblingInventoryRow, GamblingInventoryTable};
// pub use gambling_level::GamblingAndLevel;
// pub use gambling_profile::{GamblingItem, GamblingProfile};
pub use gambling_goals::GamblingGoalsRow;
pub use gambling_item::GamblingItem;
pub use game_row::{GameManager, GameRow};

use crate::{Error, FormatNum, Result, shop::ShopCurrency};

pub trait Coins {
    fn coins(&self) -> i64;

    fn coins_str(&self) -> String {
        self.coins().format()
    }

    fn coins_mut(&mut self) -> &mut i64;

    fn add_coins(&mut self, payout: i64) {
        *self.coins_mut() += payout;
    }
}

pub trait Gems {
    fn gems(&self) -> i64;

    fn gems_str(&self) -> String {
        self.gems().format()
    }

    fn gems_mut(&mut self) -> &mut i64;

    fn add_gems(&mut self, amount: i64) {
        *self.gems_mut() += amount;
    }
}

pub trait Work {
    fn work(&self) -> NaiveDateTime;

    fn update_work(&mut self);

    fn verify_work(&self) -> Result<()> {
        let now = Utc::now().naive_utc();
        let break_over = self.work() + Duration::minutes(10);

        if break_over >= now {
            return Err(Error::WorkClaimed(break_over.and_utc().timestamp()));
        }

        Ok(())
    }
}

pub trait ItemInventory {
    fn inventory(&self) -> &[GamblingItem];

    fn inventory_mut(&mut self) -> &mut Vec<GamblingItem>;

    fn edit_item_quantity(&mut self, item_id: &str, amount: i64) -> Option<i64> {
        let inv = self.inventory_mut();

        let item = inv.iter_mut().find(|item| item.item_id == item_id)?;

        item.quantity += amount;

        let quantity = item.quantity;

        if quantity == 0 {
            inv.retain(|inv_item| inv_item.item_id != item_id);
        }

        Some(quantity)
    }
}

pub trait MiningInventory {
    fn mines(&self) -> i64;

    fn land(&self) -> i64;

    fn countries(&self) -> i64;

    fn continents(&self) -> i64;

    fn planets(&self) -> i64;

    fn solar_systems(&self) -> i64;

    fn galaxies(&self) -> i64;

    fn universes(&self) -> i64;

    fn prestige(&self) -> i64;

    fn tech(&self) -> i64;

    fn utility(&self) -> i64;

    fn production(&self) -> i64;

    fn coal(&self) -> i64;

    fn iron(&self) -> i64;

    fn gold(&self) -> i64;

    fn redstone(&self) -> i64;

    fn lapis(&self) -> i64;

    fn diamonds(&self) -> i64;

    fn emeralds(&self) -> i64;

    fn max_values(&self) -> HashMap<&str, i64> {
        const MAX_MINERS_PER_MINE: u8 = 10;
        const MAX_MINES_PER_LAND: u8 = 5;
        const MAX_LAND_PER_COUNTRY: u8 = 25;
        const MAX_COUNTRIES_PER_CONTINENT: u8 = 50;
        const MAX_CONTINENTS_PER_PLANT: u8 = 7;
        const MAX_PLANTS_PER_SOLAR_SYSTEM: u8 = 8;
        const MAX_SOLAR_SYSTEM_PER_GALAXIES: u8 = 100;
        const MAX_GALAXIES_PER_UNIVERSE: u8 = u8::MAX;

        HashMap::from([
            ("miner", MAX_MINERS_PER_MINE as i64 * (self.mines() + 1)),
            ("mine", MAX_MINES_PER_LAND as i64 * (self.land() + 1)),
            ("land", MAX_LAND_PER_COUNTRY as i64 * (self.countries() + 1)),
            (
                "country",
                MAX_COUNTRIES_PER_CONTINENT as i64 * (self.continents() + 1),
            ),
            (
                "continent",
                MAX_CONTINENTS_PER_PLANT as i64 * (self.planets() + 1),
            ),
            (
                "planet",
                MAX_PLANTS_PER_SOLAR_SYSTEM as i64 * (self.solar_systems() + 1),
            ),
            (
                "solar_system",
                MAX_SOLAR_SYSTEM_PER_GALAXIES as i64 * (self.galaxies() + 1),
            ),
            (
                "galaxy",
                MAX_GALAXIES_PER_UNIVERSE as i64 * (self.universes() + 1),
            ),
            ("universe", self.prestige() + 1),
        ])
    }

    fn resources(&self) -> String {
        format!(
            "{} `{}` coal
        {} `{}` iron
        {} `{}` gold
        {} `{}` redstone
        {} `{}` lapis
        {} `{}` diamonds
        {} `{}` emeralds",
            ShopCurrency::Coal,
            self.coal().format(),
            ShopCurrency::Iron,
            self.iron().format(),
            ShopCurrency::Gold,
            self.gold().format(),
            ShopCurrency::Redstone,
            self.redstone().format(),
            ShopCurrency::Lapis,
            self.lapis().format(),
            ShopCurrency::Diamonds,
            self.diamonds().format(),
            ShopCurrency::Emeralds,
            self.emeralds().format(),
        )
    }

    fn crafted(&self) -> String {
        format!(
            "`{}` tech packs
            `{}` utility packs
            `{}` production packs",
            self.tech().format(),
            self.utility().format(),
            self.production().format()
        )
    }
}

pub trait MaxBet {
    fn level(&self) -> i32;

    fn max_bet(&self) -> i64 {
        self.level() as i64 * 10_000
    }
}

pub trait VerifyBet: Coins + MaxBet {
    fn verify_bet(&self, bet: i64) -> Result<()> {
        const MIN: i64 = 1;

        if bet < MIN {
            return Err(Error::MinimumBetAmount(MIN));
        }

        let max = self.max_bet();
        if bet > max && bet != self.coins() {
            return Err(Error::MaximumBetAmount(max));
        }

        if bet > self.coins() {
            return Err(Error::InsufficientFunds {
                required: bet - self.coins(),
                currency: ShopCurrency::Coins,
            });
        }

        Ok(())
    }
}

impl<T: Coins + MaxBet> VerifyBet for T {}

pub trait Game {
    fn game(&self) -> NaiveDateTime;

    fn verify_cooldown(&self) -> Result<()> {
        let cooldown_over = self.game() + Duration::seconds(5);

        if cooldown_over >= Utc::now().naive_utc() {
            return Err(Error::Cooldown(cooldown_over.and_utc().timestamp()));
        }

        Ok(())
    }

    fn update_game(&mut self);
}

// --- Old
// pub trait GamblingManager {
//     fn user_id(&self) -> UserId;

//     fn work_mut(&mut self) -> &mut NaiveDateTime;

//     fn update_work(&mut self) {
//         *self.work_mut() = Utc::now().naive_utc();
//     }
// }
