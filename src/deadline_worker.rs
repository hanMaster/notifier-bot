use crate::config::config;
use crate::model::deadline::search_deadline;
use cron::Schedule;
use log::{debug, error};
use sqlx::types::chrono::Local;
use std::str::FromStr;
use teloxide::prelude::Requester;
use teloxide::types::ChatId;
use teloxide::Bot;
use tokio::time::sleep;
use crate::model::stat::send_stat;

pub fn do_work(bot: Bot) {
    tokio::spawn(async move {
        let schedule =
            Schedule::from_str(&config().DEADLINE_SCHEDULE).expect("Schedule is not valid");
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
                    "{}: поиск объектов deadline",
                    Local::now().format("%d.%m.%Y %H:%M:%S")
                );
                debug!("{}", info);
                let admin_id = ChatId(config().ADMIN_ID);
                bot.send_message(admin_id, info)
                    .await
                    .expect("Unable to send message to admin");

                // Stat
                let results = send_stat().await;

                if let Err(e) = results {
                    let msg = format!("Unable to search for deadline: {}", e);
                    error!("{msg}");
                    bot.send_message(admin_id, msg)
                        .await
                        .expect("Unable to send message to admin");
                }

                // Deadline
                let results = search_deadline().await;

                if let Err(e) = results {
                    let msg = format!("Unable to search for deadline: {}", e);
                    error!("{msg}");
                    bot.send_message(admin_id, msg)
                        .await
                        .expect("Unable to send message to admin");
                }
            }
        }
    });
}
