use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;

use futures::StreamExt;
use rand::rng;
use rand::seq::{IndexedRandom, IteratorRandom};
use serenity::all::{
    Colour, CommandInteraction, ComponentInteraction, Context, CreateButton, CreateCommand,
    CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse, EmojiId, parse_emoji,
};
use sqlx::{Database, Pool};
use zayden_core::FormatNum;

use crate::events::{Dispatch, Event, GameEndEvent};
use crate::{
    CLUBS_2, CLUBS_3, CLUBS_4, CLUBS_5, CLUBS_6, CLUBS_7, CLUBS_8, CLUBS_9, CLUBS_10, CLUBS_A,
    CLUBS_J, CLUBS_K, CLUBS_Q, Coins, DIAMONDS_2, DIAMONDS_3, DIAMONDS_4, DIAMONDS_5, DIAMONDS_6,
    DIAMONDS_7, DIAMONDS_8, DIAMONDS_9, DIAMONDS_10, DIAMONDS_A, DIAMONDS_J, DIAMONDS_K,
    DIAMONDS_Q, Error, GameCache, GameManager, GameRow, GoalsManager, HEARTS_2, HEARTS_3, HEARTS_4,
    HEARTS_5, HEARTS_6, HEARTS_7, HEARTS_8, HEARTS_9, HEARTS_10, HEARTS_A, HEARTS_J, HEARTS_K,
    HEARTS_Q, Result, SPADES_2, SPADES_3, SPADES_4, SPADES_5, SPADES_6, SPADES_7, SPADES_8,
    SPADES_9, SPADES_10, SPADES_A, SPADES_J, SPADES_K, SPADES_Q, ShopCurrency,
};

use super::Commands;

const BUYIN: i64 = 100;

static NUM_TO_CARDS: LazyLock<HashMap<u8, [EmojiId; 4]>> = LazyLock::new(|| {
    HashMap::from([
        (1, [CLUBS_A, DIAMONDS_A, HEARTS_A, SPADES_A]),
        (2, [CLUBS_2, DIAMONDS_2, HEARTS_2, SPADES_2]),
        (3, [CLUBS_3, DIAMONDS_3, HEARTS_3, SPADES_3]),
        (4, [CLUBS_4, DIAMONDS_4, HEARTS_4, SPADES_4]),
        (5, [CLUBS_5, DIAMONDS_5, HEARTS_5, SPADES_5]),
        (6, [CLUBS_6, DIAMONDS_6, HEARTS_6, SPADES_6]),
        (7, [CLUBS_7, DIAMONDS_7, HEARTS_7, SPADES_7]),
        (8, [CLUBS_8, DIAMONDS_8, HEARTS_8, SPADES_8]),
        (9, [CLUBS_9, DIAMONDS_9, HEARTS_9, SPADES_9]),
        (10, [CLUBS_10, DIAMONDS_10, HEARTS_10, SPADES_10]),
        (11, [CLUBS_J, DIAMONDS_J, HEARTS_J, SPADES_J]),
        (12, [CLUBS_Q, DIAMONDS_Q, HEARTS_Q, SPADES_Q]),
        (13, [CLUBS_K, DIAMONDS_K, HEARTS_K, SPADES_K]),
    ])
});

impl Commands {
    pub async fn higher_lower<
        Db: Database,
        GoalsHandler: GoalsManager<Db>,
        GameHandler: GameManager<Db>,
    >(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer(ctx).await.unwrap();

        let mut row = GameHandler::row(pool, interaction.user.id)
            .await
            .unwrap()
            .unwrap_or_else(|| GameRow::new(interaction.user.id));

        if row.coins() < BUYIN {
            return Err(Error::InsufficientFunds {
                required: BUYIN - row.coins(),
                currency: ShopCurrency::Coins,
            });
        }

        GameCache::can_play(ctx, interaction.user.id).await?;

        *row.coins_mut() -= BUYIN;

        GameHandler::save(pool, row).await.unwrap();

        let (&num, emojis) = NUM_TO_CARDS.iter().choose(&mut rng()).unwrap();
        let &emoji = emojis.choose(&mut rng()).unwrap();

        let embed = create_embed(&format!("<:{num}:{emoji}>"), -BUYIN, true);

        let higher_btn = CreateButton::new("hol_higher").emoji('☝').label("Higher");
        let lower_btn = CreateButton::new("hol_lower").emoji('👇').label("Lower");

        let msg = interaction
            .edit_response(
                ctx,
                EditInteractionResponse::new()
                    .embed(embed)
                    .button(higher_btn)
                    .button(lower_btn),
            )
            .await
            .unwrap();

        let mut stream = msg
            .await_component_interactions(ctx)
            .author_id(interaction.user.id)
            .timeout(Duration::from_secs(120))
            .stream();

        let mut payout = -BUYIN;
        let mut prev_seq = String::new();

        while let Some(interaction) = stream.next().await {
            let mut desc_iter = interaction
                .message
                .embeds
                .first()
                .unwrap()
                .description
                .as_deref()
                .unwrap()
                .split("\n\n");

            prev_seq = desc_iter.next().unwrap().to_string();
            let prev_emoji = parse_emoji(prev_seq.split(' ').next_back().unwrap()).unwrap();
            let prev_num = prev_emoji.name.parse::<u8>().unwrap();

            let (num, emoji) = {
                let (&num, emojis) = NUM_TO_CARDS
                    .iter()
                    .filter(|(n, _)| **n != prev_num)
                    .choose(&mut rng())
                    .unwrap();
                let &emoji = emojis.choose(&mut rng()).unwrap();
                (num, emoji)
            };

            payout = desc_iter
                .next()
                .unwrap()
                .strip_prefix("Current Payout: ")
                .unwrap()
                .parse::<i64>()
                .unwrap();

            let choice = interaction.data.custom_id.as_str();

            let winner = if choice == "hol_higher" {
                higher(
                    ctx,
                    &interaction,
                    &mut prev_seq,
                    prev_num,
                    num,
                    emoji,
                    payout,
                )
                .await?
            } else {
                lower(
                    ctx,
                    &interaction,
                    &mut prev_seq,
                    prev_num,
                    num,
                    emoji,
                    payout,
                )
                .await?
            };

            if !winner {
                break;
            }
        }

        let mut row = GameHandler::row(pool, interaction.user.id)
            .await
            .unwrap()
            .unwrap();

        row.add_coins(payout + BUYIN);

        let colour = if payout > 0 {
            Colour::DARK_GREEN
        } else {
            Colour::RED
        };

        let coins = row.coins_str();

        Dispatch::<Db, GoalsHandler>::new(pool)
            .fire(
                &mut row,
                Event::GameEnd(GameEndEvent::new(
                    "higherorlower",
                    interaction.user.id,
                    payout + BUYIN,
                )),
            )
            .await?;

        GameHandler::save(pool, row).await.unwrap();
        GameCache::update(ctx, interaction.user.id).await;

        let result = if payout > 0 {
            format!("Profit: {}", payout.format())
        } else {
            format!("Lost: {}", payout.format())
        };

        let embed = CreateEmbed::new()
            .title("Higher or Lower")
            .description(format!(
                "{}\n\nFinal Payout: {}\n\nThis game has ended.\n\n{result}\nYour coins: {coins}",
                prev_seq,
                payout.format()
            ))
            .colour(colour);

        interaction
            .edit_response(
                ctx,
                EditInteractionResponse::new()
                    .embed(embed)
                    .components(Vec::new()),
            )
            .await
            .unwrap();

        Ok(())
    }

    pub fn register_higher_lower() -> CreateCommand {
        CreateCommand::new("higherorlower").description("Play a game of higher or lower")
    }
}

fn create_embed(seq: &str, payout: i64, winner: bool) -> CreateEmbed {
    let desc = if winner {
        format!("{seq}\n\nCurrent Payout: {payout}\n\nGuess the next number!")
    } else {
        format!("{seq}\n\nFinal Payout: {payout}")
    };

    CreateEmbed::new()
        .title("Higher or Lower")
        .description(desc)
        .colour(Colour::TEAL)
}

async fn higher(
    ctx: &Context,
    interaction: &ComponentInteraction,
    seq: &mut String,
    prev: u8,
    next: u8,
    emoji: EmojiId,
    mut payout: i64,
) -> Result<bool> {
    seq.push(' ');

    let winner = next > prev;

    if winner {
        seq.push('☝');
        payout += 100
    } else {
        seq.push('❌');
    }

    seq.push_str(&format!(" <:{next}:{emoji}>"));

    let embed = create_embed(seq, payout, winner);

    let msg = if winner {
        CreateInteractionResponseMessage::new().embed(embed)
    } else {
        CreateInteractionResponseMessage::new()
            .embed(embed)
            .components(Vec::new())
    };

    interaction
        .create_response(ctx, CreateInteractionResponse::UpdateMessage(msg))
        .await?;

    Ok(winner)
}

async fn lower(
    ctx: &Context,
    interaction: &ComponentInteraction,
    seq: &mut String,
    prev: u8,
    next: u8,
    emoji: EmojiId,
    mut payout: i64,
) -> Result<bool> {
    seq.push(' ');

    let winner = next < prev;

    if winner {
        seq.push('👇');
        payout += 100
    } else {
        seq.push('❌');
    }

    seq.push_str(&format!(" <:{next}:{emoji}>"));

    let embed = create_embed(seq, payout, winner);

    let msg = if winner {
        CreateInteractionResponseMessage::new().embed(embed)
    } else {
        CreateInteractionResponseMessage::new()
            .embed(embed)
            .components(Vec::new())
    };

    interaction
        .create_response(ctx, CreateInteractionResponse::UpdateMessage(msg))
        .await?;

    Ok(winner)
}
