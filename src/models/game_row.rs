use async_trait::async_trait;
use chrono::{NaiveDateTime, Utc};
use serenity::all::UserId;
use sqlx::{Database, FromRow, Pool, any::AnyQueryResult};

use super::{Coins, Game, Gems, MaxBet};

#[async_trait]
pub trait GameManager<Db: Database> {
    async fn row(pool: &Pool<Db>, id: impl Into<UserId> + Send) -> sqlx::Result<Option<GameRow>>;

    async fn save(pool: &Pool<Db>, row: GameRow) -> sqlx::Result<AnyQueryResult>;
}

#[derive(FromRow)]
pub struct GameRow {
    pub id: i64,
    pub coins: i64,
    pub gems: i64,
    pub game: NaiveDateTime,
    pub level: Option<i32>,
}

impl GameRow {
    pub fn new(id: impl Into<UserId>) -> Self {
        let id = id.into();

        Self {
            id: id.get() as i64,
            coins: 0,
            gems: 0,
            game: NaiveDateTime::default(),
            level: Some(0),
        }
    }
}

impl Coins for GameRow {
    fn coins(&self) -> i64 {
        self.coins
    }

    fn coins_mut(&mut self) -> &mut i64 {
        &mut self.coins
    }
}

impl Gems for GameRow {
    fn gems(&self) -> i64 {
        self.gems
    }

    fn gems_mut(&mut self) -> &mut i64 {
        &mut self.gems
    }
}

impl Game for GameRow {
    fn game(&self) -> chrono::NaiveDateTime {
        self.game
    }

    fn update_game(&mut self) {
        self.game = Utc::now().naive_utc()
    }
}

impl MaxBet for GameRow {
    fn level(&self) -> i32 {
        self.level.unwrap_or_default()
    }
}
