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
        match self {
            Self::GameEnd(event) => event.user_id,
            _ => todo!(),
        }
    }
}

pub struct GameEndEvent {
    pub game_id: String,
    pub user_id: UserId,
    pub payout: i64,
}

impl GameEndEvent {
    pub fn new(id: impl Into<String>, user_id: impl Into<UserId>, payout: i64) -> Self {
        Self {
            game_id: id.into(),
            user_id: user_id.into(),
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
