use std::collections::HashMap;

use chrono::{Duration, NaiveDateTime, Utc};

mod gambling_effects;
mod gambling_goals;
mod gambling_item;
mod game_row;

pub use gambling_effects::{EffectsManager, EffectsRow};
pub use gambling_goals::GamblingGoalsRow;
pub use gambling_item::GamblingItem;
pub use game_row::{GameManager, GameRow};
use sqlx::Database;
use zayden_core::FormatNum;

use crate::{Error, Result, StaminaCron, StaminaManager, shop::ShopCurrency};

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

pub trait Stamina {
    const MAX_STAMINA: i32 = 3;

    fn stamina(&self) -> i32;

    fn stamina_str(&self, timestamp: i64) -> String {
        format!(
            "{}{}",
            "🟩 ".repeat(self.stamina() as usize),
            "⬜ ".repeat((Self::MAX_STAMINA - self.stamina()) as usize)
        )
    }

    fn stamina_mut(&mut self) -> &mut i32;

    fn done_work(&mut self) {
        *self.stamina_mut() -= 1
    }

    fn verify_work<Db: Database, Manager: StaminaManager<Db>>(&self) -> Result<i64> {
        let next_timestamp = StaminaCron::cron_job::<Db, Manager>()
            .schedule
            .upcoming(chrono::Utc)
            .next()
            .unwrap_or_default()
            .timestamp();

        if self.stamina() <= 0 {
            return Err(Error::WorkClaimed(next_timestamp));
        }

        Ok(next_timestamp)
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

pub trait Mining {
    fn miners(&self) -> i64;

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

    fn units(&self) -> String {
        let max_values = self.max_values();

        let max_miners = max_values.get("miner").unwrap();
        let max_mines = max_values.get("mine").unwrap();
        let max_land = max_values.get("land").unwrap();
        let max_countries = max_values.get("country").unwrap();
        let max_continents = max_values.get("continent").unwrap();
        let max_planets = max_values.get("planet").unwrap();
        let max_solar_systems = max_values.get("solar_system").unwrap();
        let max_galaxies = max_values.get("galaxy").unwrap();
        let max_universes = max_values.get("universe").unwrap();

        format!(
            "`{}/{max_miners}` miners
        `{}/{max_mines}` mines
        `{}/{max_land}` plots of land
        `{}/{max_countries}` countries
        `{}/{max_continents}` continents
        `{}/{max_planets}` planets
        `{}/{max_solar_systems}` solar systems
        `{}/{max_galaxies}` galaxies
        `{}/{max_universes}` universes",
            self.miners(),
            self.mines(),
            self.land(),
            self.countries(),
            self.continents(),
            self.planets(),
            self.solar_systems(),
            self.galaxies(),
            self.universes(),
        )
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

    fn hourly(&self) -> i64 {
        if self.miners() <= 0 {
            return 0;
        }

        self.miners() * 10
    }
}

pub trait MaxBet {
    fn level(&self) -> i32;

    fn max_bet(&self) -> i64 {
        (self.level() as i64 * 10_000).max(10_000)
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
}
