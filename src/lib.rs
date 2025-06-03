use chrono::Days;
use chrono::NaiveTime;
use chrono::Utc;
use serenity::all::EmojiId;
use serenity::all::UserId;

pub mod commands;
pub mod error;
pub mod events;
pub mod goals;
pub mod mine;
pub mod models;
pub mod shop;
pub mod stamina;

pub use commands::Commands;
pub use commands::goals::GoalsManager;
pub use error::Error;
use error::Result;
pub use goals::GoalHandler;
pub use mine::MineManager;
pub use models::*;
pub use shop::{SHOP_ITEMS, ShopCurrency, ShopItem, ShopPage};
pub use stamina::{StaminaCron, StaminaManager};

const SUPER_USER: UserId = UserId::new(211486447369322506);

const START_AMOUNT: i64 = 1000;

const BLANK: EmojiId = EmojiId::new(1360623141969203220);

const COIN: EmojiId = EmojiId::new(1356741391090454705);
const TAILS: EmojiId = EmojiId::new(1356741709995704600);
const GEM: char = '💎';

const COAL: EmojiId = EmojiId::new(1374524818560647240);
const IRON: EmojiId = EmojiId::new(1374524826605191280);
const GOLD: EmojiId = EmojiId::new(1374524835270623262);
const REDSTONE: EmojiId = EmojiId::new(1374524844770857062);
const LAPIS: EmojiId = EmojiId::new(1374524852517736480);
const DIAMOND: EmojiId = EmojiId::new(1374523197302505472);
const EMERALD: EmojiId = EmojiId::new(1374524807491747901);

const CLUBS_2: EmojiId = EmojiId::new(1377739116833276094);
const CLUBS_3: EmojiId = EmojiId::new(1377739126232846346);
const CLUBS_4: EmojiId = EmojiId::new(1377739136605360178);
const CLUBS_5: EmojiId = EmojiId::new(1377739145291763763);
const CLUBS_6: EmojiId = EmojiId::new(1377739151448735814);
const CLUBS_7: EmojiId = EmojiId::new(1377739157094400050);
const CLUBS_8: EmojiId = EmojiId::new(1377739165394796574);
const CLUBS_9: EmojiId = EmojiId::new(1377739172206608555);
const CLUBS_10: EmojiId = EmojiId::new(1377739178359656500);
const CLUBS_J: EmojiId = EmojiId::new(1377739192867618846);
const CLUBS_Q: EmojiId = EmojiId::new(1377739205396009070);
const CLUBS_K: EmojiId = EmojiId::new(1377739199326978241);
const CLUBS_A: EmojiId = EmojiId::new(1377739186488217860);
const DIAMONDS_2: EmojiId = EmojiId::new(1377739210852663326);
const DIAMONDS_3: EmojiId = EmojiId::new(1377739216687202456);
const DIAMONDS_4: EmojiId = EmojiId::new(1377739222844182670);
const DIAMONDS_5: EmojiId = EmojiId::new(1377739229278502964);
const DIAMONDS_6: EmojiId = EmojiId::new(1377739235863302284);
const DIAMONDS_7: EmojiId = EmojiId::new(1377739245246091494);
const DIAMONDS_8: EmojiId = EmojiId::new(1377739251600592967);
const DIAMONDS_9: EmojiId = EmojiId::new(1377739259762577458);
const DIAMONDS_10: EmojiId = EmojiId::new(1377739266225864894);
const DIAMONDS_J: EmojiId = EmojiId::new(1377739279861678230);
const DIAMONDS_Q: EmojiId = EmojiId::new(1377739294403465216);
const DIAMONDS_K: EmojiId = EmojiId::new(1377739288065609889);
const DIAMONDS_A: EmojiId = EmojiId::new(1377739272408399972);
const HEARTS_2: EmojiId = EmojiId::new(1377739301701554328);
const HEARTS_3: EmojiId = EmojiId::new(1377739316394070189);
const HEARTS_4: EmojiId = EmojiId::new(1377739325055303783);
const HEARTS_5: EmojiId = EmojiId::new(1377739333284532287);
const HEARTS_6: EmojiId = EmojiId::new(1377739340633079929);
const HEARTS_7: EmojiId = EmojiId::new(1377739347843088455);
const HEARTS_8: EmojiId = EmojiId::new(1377739356869234760);
const HEARTS_9: EmojiId = EmojiId::new(1377739365509370018);
const HEARTS_10: EmojiId = EmojiId::new(1377739371947626587);
const HEARTS_J: EmojiId = EmojiId::new(1377739388066467880);
const HEARTS_Q: EmojiId = EmojiId::new(1377739404252156035);
const HEARTS_K: EmojiId = EmojiId::new(1377739395506901053);
const HEARTS_A: EmojiId = EmojiId::new(1377739378968887367);
const SPADES_2: EmojiId = EmojiId::new(1377739414477864960);
const SPADES_3: EmojiId = EmojiId::new(1377739423726309513);
const SPADES_4: EmojiId = EmojiId::new(1377739442139299920);
const SPADES_5: EmojiId = EmojiId::new(1377739451983200357);
const SPADES_6: EmojiId = EmojiId::new(1377739469624709161);
const SPADES_7: EmojiId = EmojiId::new(1377739477765853324);
const SPADES_8: EmojiId = EmojiId::new(1377739488473649212);
const SPADES_9: EmojiId = EmojiId::new(1377739496409403522);
const SPADES_10: EmojiId = EmojiId::new(1377739504558805094);
const SPADES_J: EmojiId = EmojiId::new(1377739521868693694);
const SPADES_Q: EmojiId = EmojiId::new(1377739546799640638);
const SPADES_K: EmojiId = EmojiId::new(1377739530450239538);
const SPADES_A: EmojiId = EmojiId::new(1377739512750280925);

fn tomorrow() -> i64 {
    Utc::now()
        .checked_add_days(Days::new(1))
        .unwrap()
        .with_time(NaiveTime::MIN)
        .unwrap()
        .timestamp()
}
