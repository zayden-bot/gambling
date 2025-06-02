use serenity::all::CreateCommand;

pub mod coinflip;
pub mod craft;
pub mod daily;
pub mod dig;
pub mod gift;
pub mod goals;
pub mod higher_lower;
pub mod inventory;
// pub mod leaderboard;
pub mod lotto;
pub mod profile;
// pub mod roll;
// pub mod rps;
pub mod send;
pub mod shop;
// pub mod tictactoe;
pub mod work;

pub struct Commands;
pub use lotto::Lotto;

pub fn register() -> Vec<CreateCommand> {
    vec![
        Commands::register_coinflip(),
        Commands::register_goals(),
        Commands::register_lotto(),
        Commands::register_profile(),
        Commands::register_send(),
        Commands::register_send(),
        Commands::register_shop(),
        Commands::register_work(),
    ]
    /*
    blackjack
    connectfour
    crash
    dig
    findthelady
    gamble
    poker
    process?
    race
    roulette
    sevens
    slots
    spin

    oscar can we get bank robberies that get diff chances and payout depending on how many people and pot

    Should be possible honestly, a redeem on twitch to add coins here, just not the other way around
    */
}
