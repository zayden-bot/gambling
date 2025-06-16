use chrono::DateTime;
use chrono::Days;
use chrono::NaiveTime;
use chrono::Utc;
use serenity::all::EmojiId;
use serenity::all::UserId;

pub mod commands;
pub mod error;
pub mod events;
pub mod game_cache;
pub mod goals;
pub mod lotto;
pub mod models;
pub mod shop;
pub mod stamina;

pub use commands::Commands;
pub use commands::goals::GoalsManager;
pub use error::Error;
use error::Result;
pub use game_cache::GameCache;
pub use goals::GoalHandler;
pub use lotto::{Lotto, LottoManager, LottoRow, jackpot};
pub use models::{
    Coins, EffectsManager, EffectsRow, GamblingGoalsRow, GamblingItem, GameManager, GameRow, Gems,
    ItemInventory, MaxBet, MineHourly, Mining, Stamina, VerifyBet,
};
pub use shop::{SHOP_ITEMS, ShopCurrency, ShopItem, ShopPage};
pub use stamina::{StaminaCron, StaminaManager};

const SUPER_USER: UserId = UserId::new(211486447369322506);

const START_AMOUNT: i64 = 1000;

const BLANK: EmojiId = EmojiId::new(1360623141969203220);

const COIN: EmojiId = EmojiId::new(1383692085529415680);
const TAILS: EmojiId = EmojiId::new(1356741709995704600);
const GEM: char = '💎';

const COAL: EmojiId = EmojiId::new(1374524818560647240);
const IRON: EmojiId = EmojiId::new(1374524826605191280);
const GOLD: EmojiId = EmojiId::new(1374524835270623262);
const REDSTONE: EmojiId = EmojiId::new(1374524844770857062);
const LAPIS: EmojiId = EmojiId::new(1383692268959039609);
const DIAMOND: EmojiId = EmojiId::new(1374523197302505472);
const EMERALD: EmojiId = EmojiId::new(1374524807491747901);
const TECH: EmojiId = EmojiId::new(1384190136060874853);
const UTILITY: EmojiId = EmojiId::new(1384190129421418739);
const PRODUCTION: EmojiId = EmojiId::new(1384190122320334931);

const CLUBS_2: EmojiId = EmojiId::new(1383692579710701619);
const CLUBS_3: EmojiId = EmojiId::new(1383692586107015168);
const CLUBS_4: EmojiId = EmojiId::new(1383692592990126091);
const CLUBS_5: EmojiId = EmojiId::new(1383692599755411506);
const CLUBS_6: EmojiId = EmojiId::new(1383692606126555136);
const CLUBS_7: EmojiId = EmojiId::new(1383692612313284608);
const CLUBS_8: EmojiId = EmojiId::new(1383692618151493652);
const CLUBS_9: EmojiId = EmojiId::new(1383692624124186674);
const CLUBS_10: EmojiId = EmojiId::new(1383692630084423781);
const CLUBS_J: EmojiId = EmojiId::new(1383692641862156352);
const CLUBS_Q: EmojiId = EmojiId::new(1383692653383651348);
const CLUBS_K: EmojiId = EmojiId::new(1383692647750959247);
const CLUBS_A: EmojiId = EmojiId::new(1383692636128284793);
const DIAMONDS_2: EmojiId = EmojiId::new(1383692659939610655);
const DIAMONDS_3: EmojiId = EmojiId::new(1383692665585012827);
const DIAMONDS_4: EmojiId = EmojiId::new(1383692671968743454);
const DIAMONDS_5: EmojiId = EmojiId::new(1383692678977294367);
const DIAMONDS_6: EmojiId = EmojiId::new(1383692683972968488);
const DIAMONDS_7: EmojiId = EmojiId::new(1383692690314756197);
const DIAMONDS_8: EmojiId = EmojiId::new(1383692696077467648);
const DIAMONDS_9: EmojiId = EmojiId::new(1383692701987246080);
const DIAMONDS_10: EmojiId = EmojiId::new(1383692708194816021);
const DIAMONDS_J: EmojiId = EmojiId::new(1383692719045476453);
const DIAMONDS_Q: EmojiId = EmojiId::new(1383692730626080800);
const DIAMONDS_K: EmojiId = EmojiId::new(1383692724225572864);
const DIAMONDS_A: EmojiId = EmojiId::new(1383692713660121199);
const HEARTS_2: EmojiId = EmojiId::new(1383692735789138041);
const HEARTS_3: EmojiId = EmojiId::new(1383692742479056906);
const HEARTS_4: EmojiId = EmojiId::new(1383692748921769984);
const HEARTS_5: EmojiId = EmojiId::new(1383692755917733888);
const HEARTS_6: EmojiId = EmojiId::new(1383692761663803413);
const HEARTS_7: EmojiId = EmojiId::new(1383692768387272704);
const HEARTS_8: EmojiId = EmojiId::new(1383692773458448536);
const HEARTS_9: EmojiId = EmojiId::new(1383692779053383730);
const HEARTS_10: EmojiId = EmojiId::new(1383692785554690099);
const HEARTS_J: EmojiId = EmojiId::new(1383692806245056512);
const HEARTS_Q: EmojiId = EmojiId::new(1383692825576738986);
const HEARTS_K: EmojiId = EmojiId::new(1383692818538565642);
const HEARTS_A: EmojiId = EmojiId::new(1383692792546725908);
const SPADES_2: EmojiId = EmojiId::new(1383692832438485012);
const SPADES_3: EmojiId = EmojiId::new(1383692839799754822);
const SPADES_4: EmojiId = EmojiId::new(1383692847513079808);
const SPADES_5: EmojiId = EmojiId::new(1383692854060122152);
const SPADES_6: EmojiId = EmojiId::new(1383692860561297468);
const SPADES_7: EmojiId = EmojiId::new(1383692867775627294);
const SPADES_8: EmojiId = EmojiId::new(1383692875229040741);
const SPADES_9: EmojiId = EmojiId::new(1383692882262884372);
const SPADES_10: EmojiId = EmojiId::new(1383692888998940732);
const SPADES_J: EmojiId = EmojiId::new(1383692903976534016);
const SPADES_Q: EmojiId = EmojiId::new(1383692919646584852);
const SPADES_K: EmojiId = EmojiId::new(1383692909768871990);
const SPADES_A: EmojiId = EmojiId::new(1383692901795500062);

fn tomorrow(now: Option<DateTime<Utc>>) -> i64 {
    now.unwrap_or_else(Utc::now)
        .checked_add_days(Days::new(1))
        .unwrap()
        .with_time(NaiveTime::MIN)
        .unwrap()
        .timestamp()
}
