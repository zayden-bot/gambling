mod dispatch;

pub use dispatch::Dispatch;
use serenity::all::UserId;

use crate::{Coins, Gems, MaxBet};

pub trait EventRow: Coins + Gems + MaxBet + Send + Sync {}

impl<T: Coins + Gems + MaxBet + Send + Sync> EventRow for T {}

pub enum Event {
    GameEnd(GameEndEvent),
    ShopPurchase(ShopPurchaseEvent),
    Send(SendEvent),
    Work,
}

impl Event {
    pub fn user_id(&self) -> UserId {
        todo!()
    }

    pub fn row(&self) -> &dyn EventRow {
        todo!()
    }

    pub fn row_mut(&mut self) -> &mut dyn EventRow {
        todo!()
    }
}

pub struct GameEndEvent {
    pub game_id: String,
    pub payout: i64,
}

impl GameEndEvent {
    pub fn new(id: impl Into<String>, payout: i64) -> Self {
        Self {
            game_id: id.into(),
            payout,
        }
    }
}

pub struct ShopPurchaseEvent {
    pub item_id: String,
}

impl ShopPurchaseEvent {
    pub fn new(id: impl Into<String>) -> Self {
        Self { item_id: id.into() }
    }
}

pub struct SendEvent {
    pub amount: i64,
}

impl SendEvent {
    pub fn new(amount: i64) -> Self {
        Self { amount }
    }
}
