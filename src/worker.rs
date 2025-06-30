use crate::config::config;
use crate::model::sync::sync;
use cron::Schedule;
use log::{debug, error, info};
use sqlx::types::chrono::Local;
use std::str::FromStr;
use teloxide::prelude::Requester;
use teloxide::types::ChatId;
use teloxide::Bot;
use tokio::time::sleep;

pub fn do_work(bot: Bot) {
    tokio::spawn(async move {
        let schedule = Schedule::from_str(&config().SCHEDULE).expect("Schedule is not valid");
        debug!("Upcoming fire times:");
        for datetime in schedule.upcoming(Local).take(5) {
            debug!("-> {}", datetime);
        }

        loop {
            let now = Local::now();
            if let Some(next) = schedule.upcoming(Local).next() {
                let duration = (next - now).to_std().expect("duration cannot be negative");
                sleep(duration).await;
                let info = format!(
                    "{}: запущена синхронизация",
                    Local::now().format("%d.%m.%Y %H:%M:%S")
                );
                info!("{}", info);
                let admin_id = ChatId(config().ADMIN_ID);
                let res = bot.send_message(admin_id, info).await;
                if res.is_err() {
                    error!("Unable to send message to admin: {}", res.err().unwrap());
                }

                let results = sync(&bot).await;
                for res in results {
                    match res {
                        Ok(data) => {
                            for r in data {
                                let msg = format!("Новая продажа!:\n{}", r.to_string());
                                let res = bot
                                    .send_message(ChatId(config().TG_GROUP_ID), msg)
                                    .await;
                                if res.is_err() {
                                    error!(
                                        "Unable to send message to group: {}",
                                        res.err().unwrap()
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            let res = bot.send_message(admin_id, e.to_string()).await;
                            if res.is_err() {
                                error!(
                                    "Unable to send sync error to admin: {}",
                                    res.err().unwrap()
                                );
                            }
                        }
                    }
                }
            }
        }
    });
}
