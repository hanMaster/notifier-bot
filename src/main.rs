use crate::bot_interface::{bot_handler, BotCommand, State};
pub use crate::error::Result;
use dotenvy::dotenv;
use log::info;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dptree::deps;
use teloxide::{prelude::*, utils::command::BotCommands};
use crate::model::init_db;

mod config;
mod error;
mod model;
mod adapters;
mod worker;
mod bot_interface;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().expect("dotenv init failed");

    pretty_env_logger::init();

    init_db().await?;

    info!("Starting DKP bot...");

    let bot = Bot::from_env();
    bot.set_my_commands(BotCommand::bot_commands())
        .await
        .expect("Failed to set bot commands");

    let cloned_bot = bot.clone();

    worker::do_work(cloned_bot);

    Dispatcher::builder(bot, bot_handler())
        .dependencies(deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}









