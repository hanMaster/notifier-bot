use crate::config::config;
use crate::model::sync::sync;
use cron::Schedule;
use log::debug;
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
                debug!("{}", info);
                let admin_id = ChatId(config().ADMIN_ID);
                bot.send_message(admin_id, info)
                    .await
                    .expect("Unable to send message to admin");

                let results = sync(&bot).await;
                for res in results {
                    match res {
                        Ok(data) => {
                            for r in data {
                                bot.send_message(ChatId(config().TG_GROUP_ID), r.to_string())
                                    .await
                                    .expect("Unable to send message in group");
                            }
                        }
                        Err(e) => {
                            bot.send_message(admin_id, e.to_string())
                                .await
                                .expect("Unable to send message to admin");
                        }
                    }
                }
            }
        }
    });
}
