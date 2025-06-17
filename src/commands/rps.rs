use std::{fmt::Display, str::FromStr};

use rand::seq::IndexedRandom;
use serenity::all::{
    Colour, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbed, EditInteractionResponse, ResolvedOption, ResolvedValue,
};
use sqlx::{Database, Pool};
use zayden_core::{FormatNum, parse_options};

use crate::events::{Dispatch, Event, GameEvent};
use crate::{
    COIN, Coins, EffectsManager, GameCache, GameManager, GameRow, GoalsManager, Result, VerifyBet,
};

use super::Commands;

impl Commands {
    pub async fn rps<
        Db: Database,
        GoalHandler: GoalsManager<Db>,
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

        let Some(ResolvedValue::String(selection)) = options.remove("selection") else {
            unreachable!("selection is required")
        };
        let user_choice = selection.parse::<RPSChoice>().unwrap();

        let Some(ResolvedValue::Integer(bet)) = options.remove("bet") else {
            unreachable!("bet is required")
        };

        let mut row = GameHandler::row(pool, interaction.user.id)
            .await?
            .unwrap_or_else(|| GameRow::new(interaction.user.id));

        GameCache::can_play(ctx, interaction.user.id).await?;
        row.verify_bet(bet)?;
        row.bet(bet);

        let computer_choice = *CHOICES.choose(&mut rand::rng()).unwrap();
        let winner = user_choice.winner(&computer_choice);

        let mut payout = if winner == Some(true) {
            bet * 2
        } else if winner.is_none() {
            bet
        } else {
            0
        };

        Dispatch::<Db, GoalHandler>::new(pool)
            .fire(
                &mut row,
                Event::Game(GameEvent::new("rps", interaction.user.id, bet)),
            )
            .await?;

        payout =
            EffectsHandler::payout(pool, interaction.user.id, bet, payout, winner == Some(true))
                .await;

        row.add_coins(payout);

        let coins = row.coins();

        GameHandler::save(pool, row).await?;
        GameCache::update(ctx, interaction.user.id).await;

        let title = if winner == Some(true) {
            "Rock 🪨 Paper 🗞️ Scissors ✂ - You Won!"
        } else if winner == Some(false) {
            "Rock 🪨 Paper 🗞️ Scissors ✂ - You Lost!"
        } else {
            "Rock 🪨 Paper 🗞️ Scissors ✂ - You Tied!"
        };

        let result = format!("Payout: {bet}");

        let desc = format!(
            "Your bet: {} <:coin:{COIN}>\n\n**You picked:** {}\n**Zayden picked:** {}\n\n{result}\nYour coins: {}",
            bet.format(),
            user_choice.as_emoji(),
            computer_choice.as_emoji(),
            coins.format()
        );

        let colour = if winner == Some(true) {
            Colour::DARK_GREEN
        } else if winner == Some(false) {
            Colour::RED
        } else {
            Colour::DARKER_GREY
        };

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

    pub fn register_rps() -> CreateCommand {
        CreateCommand::new("rps")
            .description("Play a game of rock paper scissors against the bot")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "selection",
                    "Your choice of Rock, Paper or Scissors",
                )
                .required(true)
                .add_string_choice("Rock", "rock")
                .add_string_choice("Paper", "paper")
                .add_string_choice("Scissors", "scissors"),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::Integer, "bet", "The amount to bet.")
                    .required(true),
            )
    }
}

const CHOICES: [RPSChoice; 3] = [RPSChoice::Rock, RPSChoice::Paper, RPSChoice::Scissors];

#[derive(Clone, Copy, PartialEq, Eq)]
enum RPSChoice {
    Rock,
    Paper,
    Scissors,
}

impl RPSChoice {
    fn winner(&self, opponent: &Self) -> Option<bool> {
        match (self, opponent) {
            (a, b) if a == b => None,
            (Self::Rock, Self::Scissors)
            | (Self::Paper, Self::Rock)
            | (Self::Scissors, Self::Paper) => Some(true),
            _ => Some(false),
        }
    }

    fn as_emoji(&self) -> &str {
        match self {
            Self::Rock => "🪨",
            Self::Paper => "🗞️",
            Self::Scissors => "✂",
        }
    }
}

impl Display for RPSChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RPSChoice::Rock => write!(f, "Rock"),
            RPSChoice::Paper => write!(f, "Paper"),
            RPSChoice::Scissors => write!(f, "Scissors"),
        }
    }
}

impl FromStr for RPSChoice {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "rock" => Ok(Self::Rock),
            "paper" => Ok(Self::Paper),
            "scissors" => Ok(Self::Scissors),
            _ => Err(()),
        }
    }
}
