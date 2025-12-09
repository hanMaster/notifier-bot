use crate::config::config;
use log::error;
use teloxide::Bot;
use teloxide::prelude::{ChatId, Requester};

pub async fn send_msg_to_admin(bot: &Bot, msg: &str) {
    let admin_id = ChatId(config().ADMIN_ID);
    let res = bot.send_message(admin_id, msg).await;
    if res.is_err() {
        error!(
            "Unable to send message: {msg} to admin: {}",
            res.err().unwrap()
        );
    }
}

pub async fn send_msg_to_group(bot: &Bot, msg: &str) {
    let group_id = ChatId(config().TG_GROUP_ID);
    let res = bot.send_message(group_id, msg).await;
    if res.is_err() {
        error!(
            "Unable to send message: {msg} to group: {}",
            res.err().unwrap()
        );
    }
}
