use async_trait::async_trait;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::rng;
use serenity::all::{
    ChannelId, CommandInteraction, Context, CreateCommand, CreateEmbed, CreateMessage,
    EditInteractionResponse, Mentionable, UserId,
};
use sqlx::any::AnyQueryResult;
use sqlx::prelude::FromRow;
use sqlx::{Database, Pool};
use zayden_core::CronJob;

use crate::shop::LOTTO_TICKET;
use crate::{COIN, Coins, Commands, Error, FormatNum, Result};

const CHANNEL_ID: ChannelId = ChannelId::new(1281440730820116582);

#[async_trait]
pub trait LottoManager<Db: Database> {
    async fn row(
        conn: &mut Db::Connection,
        id: impl Into<UserId> + Send,
    ) -> sqlx::Result<Option<LottoRow>>;

    async fn rows(conn: &mut Db::Connection) -> sqlx::Result<Vec<LottoRow>>;

    async fn total_tickets(conn: &mut Db::Connection) -> sqlx::Result<i64>;

    async fn delete_tickets(conn: &mut Db::Connection) -> sqlx::Result<AnyQueryResult>;

    async fn save(conn: &mut Db::Connection, row: LottoRow) -> sqlx::Result<AnyQueryResult>;
}

#[derive(FromRow)]
pub struct LottoRow {
    id: i64,
    coins: i64,
    quantity: Option<i64>,
}

impl LottoRow {
    fn new(id: impl Into<UserId> + Send) -> Self {
        let id = id.into();

        Self {
            id: id.get() as i64,
            coins: 0,
            quantity: Some(0),
        }
    }

    fn user_id(&self) -> UserId {
        UserId::new(self.id as u64)
    }

    fn quantity(&self) -> i64 {
        self.quantity.unwrap_or(0)
    }
}

impl Coins for LottoRow {
    fn coins(&self) -> i64 {
        self.coins
    }

    fn coins_mut(&mut self) -> &mut i64 {
        &mut self.coins
    }
}

pub struct Lotto;

impl Lotto {
    pub fn cron_job<E: std::error::Error, Db: Database, Manager: LottoManager<Db>>()
    -> CronJob<Db, E> {
        CronJob::new("0 0 17 * * Fri *").set_action(|ctx, pool| async move {
            let mut tx: sqlx::Transaction<'static, Db> = pool.begin().await.unwrap();

            let mut rows = Manager::rows(&mut *tx).await.unwrap();

            let prize_share = [0.5, 0.3, 0.2];

            let expected_winners = prize_share.len();

            if rows.len() < expected_winners {
                return Ok(());
            }

            let total_tickets: i64 = rows.iter().map(|row| row.quantity()).sum();
            let jackpot = total_tickets * LOTTO_TICKET.coin_cost().unwrap();

            let mut dist = WeightedIndex::new(rows.iter().map(|row| row.quantity())).unwrap();

            let winners = (0..expected_winners).map(|_| {
                let index = dist.sample(&mut rng());
                let winner = rows.remove(index);
                dist = WeightedIndex::new(rows.iter().map(|row| row.quantity())).unwrap();
                winner
            });

            Manager::delete_tickets(&mut *tx).await.unwrap();

            for (mut winner, share) in winners.into_iter().zip(prize_share) {
                let payout = (jackpot as f64 * share) as i64;

                winner.add_coins(payout);
                let mention = winner.user_id().mention();

                Manager::save(&mut *tx, winner).await.unwrap();

                CHANNEL_ID
                    .send_message(
                        &ctx,
                        CreateMessage::new().content(format!(
                            "{mention} has won {} <:coin:{COIN}> from the lottery!",
                            payout.format()
                        )),
                    )
                    .await
                    .unwrap();
            }

            tx.commit().await.unwrap();

            Ok(())
        })
    }
}

impl Commands {
    pub async fn lotto<Db: Database, Manager: LottoManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer(ctx).await.unwrap();

        let mut tx = pool.begin().await.unwrap();

        let total_tickets = Manager::total_tickets(&mut tx).await.unwrap();
        let jackpot = total_tickets * LOTTO_TICKET.coin_cost().unwrap();

        let row = match Manager::row(&mut tx, interaction.user.id).await.unwrap() {
            Some(row) => row,
            None => LottoRow::new(interaction.user.id),
        };

        let lotto_emoji = LOTTO_TICKET.emoji();

        let timestamp = {
            Lotto::cron_job::<Error, Db, Manager>()
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
                format!("{} <:coin:{COIN}>", jackpot.format()),
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
