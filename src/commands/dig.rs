use std::{collections::HashMap, sync::LazyLock};

use async_trait::async_trait;
use rand::rng;
use rand_distr::{Binomial, Distribution};
use serenity::all::{
    Colour, CommandInteraction, Context, CreateCommand, CreateEmbed, EditInteractionResponse,
    UserId,
};
use sqlx::{Database, Pool, any::AnyQueryResult, prelude::FromRow};

use crate::events::{Dispatch, Event};
use crate::shop::ShopCurrency;
use crate::{Coins, Gems, GoalsManager, MaxBet, Result, Stamina, StaminaManager};

use super::Commands;

const CHUNK_BLOCKS: f64 = 16.0 * 16.0 * 123.0;
const COAL_PER_CHUNK: f64 = 141.0;
const IRON_PER_CHUNK: f64 = 77.0;
const GOLD_PER_CHUNK: f64 = 8.3;
const REDSTONE_PER_CHUNK: f64 = 7.8;
const LAPIS_PER_CHUNK: f64 = 4.3;
const DIAMOND_PER_CHUNK: f64 = 3.7;
const EMERALDS_PER_CHUNK: f64 = 3.0;

static CHANCES: LazyLock<HashMap<&str, f64>> = LazyLock::new(|| {
    HashMap::from([
        ("coal", (COAL_PER_CHUNK / CHUNK_BLOCKS) * 100.0),
        ("iron", (IRON_PER_CHUNK / CHUNK_BLOCKS) * 100.0),
        ("gold", (GOLD_PER_CHUNK / CHUNK_BLOCKS) * 100.0),
        ("redstone", (REDSTONE_PER_CHUNK / CHUNK_BLOCKS) * 100.0),
        ("lapis", (LAPIS_PER_CHUNK / CHUNK_BLOCKS) * 100.0),
        ("diamonds", (DIAMOND_PER_CHUNK / CHUNK_BLOCKS) * 100.0),
        ("emeralds", (EMERALDS_PER_CHUNK / CHUNK_BLOCKS) * 100.0),
    ])
});

#[async_trait]
pub trait DigManager<Db: Database> {
    async fn row(pool: &Pool<Db>, id: impl Into<UserId> + Send) -> sqlx::Result<Option<DigRow>>;

    async fn save(pool: &Pool<Db>, row: DigRow) -> sqlx::Result<AnyQueryResult>;
}

#[derive(Debug, FromRow)]
pub struct DigRow {
    pub id: i64,
    pub coins: i64,
    pub gems: i64,
    pub stamina: i32,
    pub level: Option<i32>,
    pub miners: i64,
    pub coal: i64,
    pub iron: i64,
    pub gold: i64,
    pub redstone: i64,
    pub lapis: i64,
    pub diamonds: i64,
    pub emeralds: i64,
}

impl DigRow {
    pub fn new(id: impl Into<UserId>) -> Self {
        let id = id.into();

        Self {
            id: id.get() as i64,
            coins: 0,
            gems: 0,
            stamina: 0,
            level: Some(0),
            miners: 0,
            coal: 0,
            iron: 0,
            gold: 0,
            redstone: 0,
            lapis: 0,
            diamonds: 0,
            emeralds: 0,
        }
    }
}

impl Coins for DigRow {
    fn coins(&self) -> i64 {
        self.coins
    }

    fn coins_mut(&mut self) -> &mut i64 {
        &mut self.coins
    }
}

impl Gems for DigRow {
    fn gems(&self) -> i64 {
        self.gems
    }

    fn gems_mut(&mut self) -> &mut i64 {
        &mut self.gems
    }
}

impl Stamina for DigRow {
    fn stamina(&self) -> i32 {
        self.stamina
    }

    fn stamina_mut(&mut self) -> &mut i32 {
        &mut self.stamina
    }
}

impl MaxBet for DigRow {
    fn level(&self) -> i32 {
        self.level.unwrap_or_default()
    }
}

impl Commands {
    pub async fn dig<
        Db: Database,
        StaminaHandler: StaminaManager<Db>,
        GoalsHandler: GoalsManager<Db>,
        DigHandler: DigManager<Db>,
    >(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer(ctx).await.unwrap();

        let mut row = DigHandler::row(pool, interaction.user.id)
            .await
            .unwrap()
            .unwrap_or_else(|| DigRow::new(interaction.user.id));

        let timestamp = row.verify_work::<Db, StaminaHandler>()?;

        let mut resources = HashMap::from([
            ("coal", 0),
            ("iron", 0),
            ("gold", 0),
            ("redstone", 0),
            ("lapis", 0),
            ("diamonds", 0),
            ("emeralds", 0),
        ]);

        let num_attempts = (row.miners as u64).saturating_add(10);

        for (resource, chance) in CHANCES.iter() {
            *resources.get_mut(resource).unwrap() += Binomial::new(num_attempts, *chance)
                .unwrap()
                .sample(&mut rng()) as i64;
        }

        resources.iter().for_each(|(&k, &v)| match k {
            "coal" => row.coal += v,
            "iron" => row.iron += v,
            "gold" => row.gold += v,
            "redstone" => row.redstone += v,
            "lapis" => row.lapis += v,
            "diamonds" => row.diamonds += v,
            "emeralds" => row.emeralds += v,
            s => unreachable!("Invalid resource: {s}"),
        });

        Dispatch::<Db, GoalsHandler>::new(pool)
            .fire(&mut row, Event::Work(interaction.user.id))
            .await?;

        row.done_work();

        let stamina = if row.stamina() == 0 {
            format!("Time for a break. Come back <t:{timestamp}:R>")
        } else {
            "⛏️ ".repeat(row.stamina() as usize)
        };

        DigHandler::save(pool, row).await.unwrap();

        let embed = CreateEmbed::new()
            .description(format!(
                "You dug around in the mines and found:\n{}\nStamina: {}",
                {
                    let found = resources
                        .drain()
                        .filter(|(_, v)| *v > 0)
                        .map(|(k, v)| match k {
                            "coal" => (ShopCurrency::Coal, v, k),
                            "iron" => (ShopCurrency::Iron, v, k),
                            "gold" => (ShopCurrency::Gold, v, k),
                            "redstone" => (ShopCurrency::Redstone, v, k),
                            "lapis" => (ShopCurrency::Lapis, v, k),
                            "diamonds" => (ShopCurrency::Diamonds, v, k),
                            "emeralds" => (ShopCurrency::Emeralds, v, k),
                            s => unreachable!("Invalid resource: {s}"),
                        })
                        .map(|(currency, amount, name)| format!("{currency} `{amount}` {name}",))
                        .collect::<Vec<_>>();

                    if found.is_empty() {
                        String::from("Just a whole lot of boring stone...")
                    } else {
                        found.join("\n")
                    }
                },
                stamina
            ))
            .color(Colour::GOLD);

        interaction
            .edit_response(ctx, EditInteractionResponse::new().embed(embed))
            .await
            .unwrap();

        Ok(())
    }

    pub fn register_dig() -> CreateCommand {
        CreateCommand::new("dig").description("Dig in the mines to collect resources")
    }
}
