use serenity::all::{
    Colour, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbed, EditInteractionResponse, ResolvedOption, ResolvedValue,
};
use sqlx::{Database, Pool};
use zayden_core::{FormatNum, parse_options};

use crate::events::{Dispatch, Event, GameEvent};
use crate::{
    COIN, Coins, EffectsManager, Error, GameCache, GameManager, GameRow, GoalsManager, Result,
    VerifyBet,
};

use super::Commands;

impl Commands {
    pub async fn roll<
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
        interaction.defer(ctx).await?;

        let mut options = parse_options(options);

        let Some(ResolvedValue::String(dice)) = options.remove("dice") else {
            unreachable!("dice option is required")
        };

        let n_sides = dice.parse::<i64>().unwrap();

        let Some(ResolvedValue::Integer(prediction)) = options.remove("prediction") else {
            unreachable!("prediction option is required")
        };

        verify_prediction(prediction, 1, n_sides)?;

        let mut row = GameHandler::row(pool, interaction.user.id)
            .await
            .unwrap()
            .unwrap_or_else(|| GameRow::new(interaction.user.id));

        GameCache::can_play(ctx, interaction.user.id).await?;

        let Some(ResolvedValue::Integer(bet)) = options.remove("bet") else {
            unreachable!("bet option is required")
        };

        row.verify_bet(bet)?;

        let roll = rand::random_range(1..=n_sides);

        let (title, result, mut payout, colour) = if roll == prediction {
            (
                "🎲 Dice Roll 🎲 - You Won!",
                "Profit:",
                bet * n_sides,
                Colour::DARK_GREEN,
            )
        } else {
            ("🎲 Dice Roll 🎲 - You Lost!", "Lost:", -bet, Colour::RED)
        };

        Dispatch::<Db, GoalHandler>::new(pool)
            .fire(
                &mut row,
                Event::Game(GameEvent::new("roll", interaction.user.id, bet)),
            )
            .await?;

        payout = EffectsHandler::payout(pool, interaction.user.id, payout).await;
        payout -= bet;

        row.add_coins(payout);

        let coins = row.coins();

        GameHandler::save(pool, row).await.unwrap();
        GameCache::update(ctx, interaction.user.id).await;

        let desc = format!(
            "Your bet: {} <:coin:{COIN}>\n\n**You picked:** {prediction} 🎲\n**Result:** {roll} 🎲\n\n{result} {}\nYour coins: {}",
            bet.format(),
            payout.format(),
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

    pub fn register_roll() -> CreateCommand {
        CreateCommand::new("roll")
            .description("Roll the dice")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "dice",
                    "The type of dice to roll",
                )
                .add_string_choice("4-sides", "4")
                .add_string_choice("6-sides", "6")
                .add_string_choice("8-sides", "8")
                .add_string_choice("10-sides", "10")
                .add_string_choice("12-sides", "12")
                .add_string_choice("20-sides", "20")
                .required(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "prediction",
                    "What number will the dice land on?",
                )
                .required(true),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::Integer, "bet", "Roll the dice")
                    .required(true),
            )
    }
}

fn verify_prediction(prediction: i64, min: i64, max: i64) -> Result<()> {
    if prediction > max || prediction < min {
        return Err(Error::InvalidPrediction);
    }

    Ok(())
}
