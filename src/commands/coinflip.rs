use std::fmt::Display;
use std::str::FromStr;

use serenity::all::{
    Colour, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbed, EditInteractionResponse, ResolvedOption, ResolvedValue,
};
use sqlx::{Database, Pool};
use zayden_core::{FormatNum, parse_options};

use crate::events::{Dispatch, Event, GameEvent};
use crate::{
    COIN, Coins, EffectsManager, GameCache, GameManager, GameRow, GoalsManager, Result, TAILS,
    VerifyBet,
};

use super::Commands;

impl Commands {
    pub async fn coinflip<
        Db: Database,
        GoalsHandler: GoalsManager<Db>,
        EffectsHandler: EffectsManager<Db> + Send,
        GameHandler: GameManager<Db>,
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

        let mut row = GameHandler::row(pool, interaction.user.id)
            .await?
            .unwrap_or_else(|| GameRow::new(interaction.user.id));

        GameCache::can_play(ctx, interaction.user.id).await?;
        row.verify_bet(bet)?;

        let heads = rand::random_bool(0.5);
        let mut payout = bet;
        let winner = matches!(prediction, CoinSide::Heads) && heads;

        if rand::random_bool(1.0 / 6000.0) {
            payout *= 1000;
        } else if !winner {
            payout = -payout;
        }

        Dispatch::<Db, GoalsHandler>::new(pool)
            .fire(
                &mut row,
                Event::Game(GameEvent::new("coinflip", interaction.user.id, payout)),
            )
            .await?;

        payout = EffectsHandler::payout(pool, interaction.user.id, payout).await;

        row.add_coins(payout);

        let coins = row.coins();

        GameHandler::save(pool, row).await.unwrap();
        GameCache::update(ctx, interaction.user.id).await;

        let (coin, title) = if payout == bet * 1000 {
            (prediction, "Coin Flip - EDGE ROLL!")
        } else if winner {
            (prediction, "Coin Flip - You Won!")
        } else {
            (prediction.opposite(), "Coin Flip - You Lost!")
        };

        let (result, colour) = if winner {
            (format!("Profit: {}", payout.format()), Colour::DARK_GREEN)
        } else {
            (format!("Lost: {}", payout.format()), Colour::RED)
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
