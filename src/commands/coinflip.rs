use std::fmt::Display;
use std::str::FromStr;

use async_trait::async_trait;
use chrono::{NaiveDateTime, Utc};
use serenity::all::{
    Colour, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbed, EditInteractionResponse, ResolvedOption, ResolvedValue, UserId,
};
use sqlx::any::AnyQueryResult;
use sqlx::prelude::FromRow;
use sqlx::{Database, Pool};
use zayden_core::parse_options;

use crate::events::{Dispatch, Event, GameEndEvent};
use crate::{
    COIN, Coins, EffectsManager, FormatNum, Game, GoalsManager, MaxBet, Result, TAILS, VerifyBet,
};

use super::Commands;

#[async_trait]
pub trait CoinflipManager<Db: Database> {
    async fn row(
        pool: &Pool<Db>,
        id: impl Into<UserId> + Send,
    ) -> sqlx::Result<Option<CoinflipRow>>;

    async fn save(pool: &Pool<Db>, row: CoinflipRow) -> sqlx::Result<AnyQueryResult>;
}

#[derive(FromRow)]
pub struct CoinflipRow {
    pub id: i64,
    pub coins: i64,
    pub game: NaiveDateTime,
    pub level: i32,
}

impl CoinflipRow {
    pub fn new(id: impl Into<UserId>) -> Self {
        let id = id.into();

        Self {
            id: id.get() as i64,
            coins: 0,
            game: NaiveDateTime::default(),
            level: 0,
        }
    }
}

impl Coins for CoinflipRow {
    fn coins(&self) -> i64 {
        self.coins
    }

    fn coins_mut(&mut self) -> &mut i64 {
        &mut self.coins
    }
}

impl Game for CoinflipRow {
    fn game(&self) -> chrono::NaiveDateTime {
        self.game
    }

    fn update_game(&mut self) {
        self.game = Utc::now().naive_utc()
    }
}

impl MaxBet for CoinflipRow {
    fn level(&self) -> i32 {
        self.level
    }
}

impl Commands {
    pub async fn coinflip<
        Db: Database,
        GoalsHandler: GoalsManager<Db>,
        EffectsHandler: EffectsManager<Db> + Send,
        CoinflipHandler: CoinflipManager<Db>,
    >(
        ctx: &Context,
        interaction: &CommandInteraction,
        options: Vec<ResolvedOption<'_>>,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer(ctx).await.unwrap();

        let mut options = parse_options(options);

        let Some(ResolvedValue::String(prediction)) = options.remove("prediction") else {
            unreachable!("prediction is required")
        };
        let prediction = prediction.parse::<CoinSide>().unwrap();

        let Some(ResolvedValue::Integer(bet)) = options.remove("bet") else {
            unreachable!("bet is required")
        };

        let mut row = CoinflipHandler::row(pool, interaction.user.id)
            .await?
            .unwrap_or_else(|| CoinflipRow::new(interaction.user.id));

        row.verify_cooldown()?;
        row.verify_bet(bet)?;

        let mut payout = bet;
        let winner = rand::random_bool(0.5);

        if winner && rand::random_bool(1.0 / 6000.0) {
            payout *= 5000;
        } else if !winner {
            payout = -payout;
        }

        payout = EffectsHandler::payout(pool, interaction.user.id, payout).await;

        row.add_coins(payout);

        let coins = row.coins();

        Dispatch::<Db, GoalsHandler>::new(pool)
            .fire(Event::GameEnd(GameEndEvent::new("coinflip", payout)))
            .await?;

        let (coin, title) = if winner {
            (prediction, "Coin Flip - You Won!")
        } else {
            (prediction.opposite(), "Coin Flip - You Lost!")
        };

        let (result, colour) = if winner {
            (format!("Profit: {payout}"), Colour::DARK_GREEN)
        } else {
            (format!("Lost: {payout}"), Colour::RED)
        };

        let desc = format!(
            "Your bet: {} <:coin:{COIN}>\n\n**You bet on:** {} ({prediction})\n**Coin landed on:** {} ({coin})\n\n{result}\nYour coins: {}",
            bet.format(),
            prediction.as_emoji(),
            coin.as_emoji(),
            coins.format()
        );

        let embed = CreateEmbed::new()
            .title(title)
            .description(desc)
            .colour(colour);

        interaction
            .edit_response(ctx, EditInteractionResponse::new().embed(embed))
            .await
            .unwrap();

        Ok(())
    }

    pub fn register_coinflip() -> CreateCommand {
        CreateCommand::new("coinflip")
            .description("Flip a coin!")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "prediction",
                    "Choose whether you think the coin will be heads or tails",
                )
                .add_string_choice("Heads", "heads")
                .add_string_choice("Tails", "tails")
                .required(true),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::Integer, "bet", "The amount to bet.")
                    .required(true),
            )
    }
}

#[derive(Debug, Clone, Copy)]
enum CoinSide {
    Heads,
    Tails,
}

impl CoinSide {
    fn opposite(&self) -> CoinSide {
        match self {
            CoinSide::Heads => CoinSide::Tails,
            CoinSide::Tails => CoinSide::Heads,
        }
    }

    fn as_emoji(&self) -> String {
        match self {
            CoinSide::Heads => format!("<:heads:{COIN}>"),
            CoinSide::Tails => format!("<:tails:{TAILS}>"),
        }
    }
}

impl Display for CoinSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoinSide::Heads => write!(f, "Heads"),
            CoinSide::Tails => write!(f, "Tails"),
        }
    }
}

impl FromStr for CoinSide {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "heads" => Ok(CoinSide::Heads),
            "tails" => Ok(CoinSide::Tails),
            _ => Err(()),
        }
    }
}
