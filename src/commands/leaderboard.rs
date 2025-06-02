use std::time::Duration;

use async_trait::async_trait;
use futures::{StreamExt, stream};
use serenity::all::{
    Colour, CommandInteraction, CommandOptionType, ComponentInteraction, Context, CreateButton,
    CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse,
    CreateInteractionResponseMessage, EditInteractionResponse, Mentionable, Message,
    ResolvedOption, ResolvedValue, UserId,
};
use zayden_core::SlashCommand;

use crate::{Error, Result};

pub struct Leaderboard;

#[async_trait]
impl SlashCommand<Error, Postgres> for Leaderboard {
    async fn run(
        ctx: &Context,
        interaction: &CommandInteraction,
        mut options: Vec<ResolvedOption<'_>>,
        pool: &PgPool,
    ) -> Result<()> {
        interaction.defer(ctx).await.unwrap();

        let ResolvedValue::String(leaderboard) = options.pop().unwrap().value else {
            unreachable!("leaderboard option is required")
        };

        let users = {
            let data = ctx.data.read().await;
            let cache = data.get::<GuildMembersCache>().unwrap();
            cache
                .get(&interaction.guild_id.unwrap())
                .unwrap()
                .iter()
                .map(|id| id.get() as i64)
                .collect::<Vec<_>>()
        };

        let rows = get_rows(leaderboard, pool, &users, 1).await;

        let desc = rows
            .into_iter()
            .enumerate()
            .map(|(i, row)| row.as_desc(i))
            .collect::<Vec<_>>()
            .join("\n\n");

        let embed = CreateEmbed::new()
            .title(format!("🏁 Leaderboard ({leaderboard})"))
            .description(desc)
            .footer(CreateEmbedFooter::new("Page 1"))
            .colour(Colour::TEAL);

        let mut response = EditInteractionResponse::new()
            .embed(embed)
            .button(CreateButton::new("leaderboard_previous").label("<"));

        if get_row_number(leaderboard, pool, interaction.user.id)
            .await
            .is_some()
        {
            response = response.button(CreateButton::new("leaderboard_user").emoji('🎯'));
        }

        let msg = interaction
            .edit_response(
                ctx,
                response.button(CreateButton::new("leaderboard_next").label(">")),
            )
            .await
            .unwrap();

        let mut stream = msg
            .await_component_interactions(ctx)
            .timeout(Duration::from_secs(120))
            .stream();

        while let Some(component) = stream.next().await {
            run_component(ctx, pool, &users, &msg, component).await?;
        }

        interaction
            .edit_response(ctx, EditInteractionResponse::new().components(Vec::new()))
            .await?;

        Ok(())
    }

    fn register(_ctx: &Context) -> Result<CreateCommand> {
        let cmd = CreateCommand::new("leaderboard")
            .description("The server leaderboard")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "leaderboard",
                    "The leaderboard to choose",
                )
                .required(true)
                .add_string_choice("Level", "level")
                .add_string_choice("Net Worth", "networth")
                .add_string_choice("Coins", "coins")
                .add_string_choice("Gem", "gem")
                .add_string_choice(EGGPLANT.name, "eggplant")
                .add_string_choice(LOTTO_TICKET.name, "lottoticket"),
            );

        Ok(cmd)
    }
}

async fn run_component(
    ctx: &Context,
    pool: &PgPool,
    users: &[i64],
    msg: &Message,
    interaction: ComponentInteraction,
) -> Result<()> {
    let custom_id = interaction
        .data
        .custom_id
        .strip_prefix("leaderboard_")
        .unwrap();

    let embed = msg.embeds.first().unwrap();

    let leaderboard = embed
        .title
        .as_ref()
        .unwrap()
        .strip_prefix("🏁 Leaderboard (")
        .unwrap()
        .strip_suffix(")")
        .unwrap();

    let mut page_number: i64 = embed
        .footer
        .as_ref()
        .unwrap()
        .text
        .strip_prefix("Page ")
        .unwrap()
        .parse()
        .unwrap();

    let rows = match custom_id {
        "previous" => {
            page_number = (page_number - 1).max(1);

            get_rows(leaderboard, pool, users, page_number).await
        }
        "user" => {
            let row_num = get_row_number(leaderboard, pool, interaction.user.id)
                .await
                .unwrap();
            page_number = row_num / 10 + 1;

            get_rows(leaderboard, pool, users, page_number).await
        }
        "next" => {
            page_number += 1;

            get_rows(leaderboard, pool, users, page_number).await
        }
        _ => unreachable!("Invalid custom id"),
    };

    let desc = rows
        .into_iter()
        .enumerate()
        .map(|(i, row)| row.as_desc(i + (page_number as usize - 1) * 10))
        .collect::<Vec<_>>()
        .join("\n\n");

    let embed = CreateEmbed::from(embed.clone())
        .footer(CreateEmbedFooter::new(format!("Page {}", page_number)))
        .description(desc);

    interaction
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new().embed(embed),
            ),
        )
        .await
        .unwrap();

    Ok(())
}

enum LeaderboardRow {
    Level(LevelsRow),
    NetWorth(GamblingRow, Vec<GamblingInventoryRow>),
    Coins(GamblingRow),
    Gem(GamblingRow),
    Eggplant(GamblingInventoryRow),
    LottoTicket(GamblingInventoryRow),
}

impl LeaderboardRow {
    pub fn user_id(&self) -> UserId {
        match self {
            Self::Level(row) => UserId::new(row.id as u64),
            Self::NetWorth(row, _) => row.user_id(),
            Self::Coins(row) => row.user_id(),
            Self::Gem(row) => row.user_id(),
            Self::Eggplant(row) => row.user_id(),
            Self::LottoTicket(row) => row.user_id(),
        }
    }

    pub fn as_desc(&self, i: usize) -> String {
        let place = if i == 0 {
            "🥇".to_string()
        } else if i == 1 {
            "🥈".to_string()
        } else if i == 2 {
            "🥉".to_string()
        } else {
            format!("#{}", i + 1)
        };

        let data = match self {
            Self::Level(row) => format!(
                "{}\n(Messages: {} | Total XP: {})",
                row.level(),
                row.message_count(),
                row.xp(),
            ),
            Self::NetWorth(row, inv) => {
                let items: u64 = inv
                    .iter()
                    .map(|inv_item| {
                        let item = SHOP_ITEMS.get(&inv_item.item_id).unwrap();
                        item.coin_cost().unwrap() as u64 * inv_item.quantity as u64
                    })
                    .sum();

                (row.coins() as u64 + items).to_string()
            }
            Self::Coins(row) => row.coins().to_string(),
            Self::Gem(row) => row.gems().to_string(),
            Self::Eggplant(row) => format!("{} {}", row.quantity, EGGPLANT.emoji()),
            Self::LottoTicket(row) => format!("{} {}", row.quantity, LOTTO_TICKET.emoji()),
        };

        format!("{place} - {} - {data}", self.user_id().mention())
    }
}

async fn get_rows(
    leaderboard: &str,
    pool: &PgPool,
    users: &[i64],
    page_num: i64,
) -> Vec<LeaderboardRow> {
    match leaderboard {
        "level" => LevelsTable::leaderboard(pool, users, page_num)
            .await
            .unwrap()
            .into_iter()
            .map(LeaderboardRow::Level)
            .collect(),
        "networth" => {
            let rows = GamblingTable::leaderboard(pool, users, "cash", page_num)
                .await
                .unwrap();
            stream::iter(rows)
                .then(|row| async {
                    let inv = GamblingInventoryTable::get_user(pool, row.user_id())
                        .await
                        .unwrap();
                    LeaderboardRow::NetWorth(row, inv)
                })
                .collect()
                .await
        }
        "coins" => GamblingTable::leaderboard(pool, users, "cash", page_num)
            .await
            .unwrap()
            .into_iter()
            .map(LeaderboardRow::Coins)
            .collect(),
        "gem" => GamblingTable::leaderboard(pool, users, "diamonds", page_num)
            .await
            .unwrap()
            .into_iter()
            .map(LeaderboardRow::Gem)
            .collect(),
        "eggplant" => GamblingInventoryTable::leaderboard(pool, "eggplant", users, page_num)
            .await
            .unwrap()
            .into_iter()
            .map(LeaderboardRow::Eggplant)
            .collect(),
        "lottoticket" => GamblingInventoryTable::leaderboard(pool, "lottoticket", users, page_num)
            .await
            .unwrap()
            .into_iter()
            .map(LeaderboardRow::LottoTicket)
            .collect(),
        _ => unreachable!("Invalid leaderboard option"),
    }
}

async fn get_row_number(leaderboard: &str, pool: &PgPool, user: UserId) -> Option<i64> {
    match leaderboard {
        "level" => LevelsTable::user_row_number(pool, user).await.ok(),

        "coins" => GamblingTable::user_row_number(pool, user, "cash")
            .await
            .ok(),
        "gem" => GamblingTable::user_row_number(pool, user, "diamonds")
            .await
            .ok(),
        "eggplant" => GamblingInventoryTable::user_row_number(pool, user, "eggplant")
            .await
            .ok(),
        "lottoticket" => GamblingInventoryTable::user_row_number(pool, user, "lottoticket")
            .await
            .ok(),
        _ => None,
    }
}
