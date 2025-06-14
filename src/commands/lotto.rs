use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateEmbed, EditInteractionResponse,
};
use sqlx::{Database, Pool};
use zayden_core::FormatNum;

use crate::shop::LOTTO_TICKET;
use crate::{COIN, Commands, Lotto, LottoManager, LottoRow, Result, jackpot};

impl Commands {
    pub async fn lotto<Db: Database, Manager: LottoManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer(ctx).await.unwrap();

        let mut tx = pool.begin().await.unwrap();

        let total_tickets = Manager::total_tickets(&mut tx).await.unwrap();

        let row = match Manager::row(&mut tx, interaction.user.id).await.unwrap() {
            Some(row) => row,
            None => LottoRow::new(interaction.user.id),
        };

        let lotto_emoji = LOTTO_TICKET.emoji();

        let timestamp = {
            Lotto::cron_job::<Db, Manager>()
                .schedule
                .upcoming(chrono::Utc)
                .next()
                .unwrap_or_default()
                .timestamp()
        };

        let embed = CreateEmbed::new()
            .title(format!(
                "<:coin:{COIN}> <:coin:{COIN}> Lottery!! <:coin:{COIN}> <:coin:{COIN}>"
            ))
            .description(format!("Draws are at <t:{timestamp}:F>"))
            .field(
                "Tickets Bought",
                format!("{} {lotto_emoji}", total_tickets.format()),
                false,
            )
            .field(
                "Jackpot Value",
                format!("{} <:coin:{COIN}>", jackpot(total_tickets).format()),
                false,
            )
            .field(
                "Your Tickets",
                format!("{} {lotto_emoji}", row.quantity().format()),
                false,
            );

        interaction
            .edit_response(ctx, EditInteractionResponse::new().embed(embed))
            .await
            .unwrap();

        Ok(())
    }

    pub fn register_lotto() -> CreateCommand {
        CreateCommand::new("lotto").description("Show the lottery information")
    }
}
