use crate::config::config;
use crate::model::sync::sync;
use crate::sender::{send_msg_to_admin, send_msg_to_group};
use cron::Schedule;
use log::{debug, info};
use sqlx::types::chrono::Local;
use std::str::FromStr;
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
                info!("{info}");
                send_msg_to_admin(&bot, &info).await;

                let results = sync(&bot).await;
                for res in results {
                    match res {
                        Ok(data) => {
                            for r in data {
                                let msg = format!("Новая продажа!\n{r}");
                                send_msg_to_group(&bot, &msg).await;
                            }
                        }
                        Err(e) => {
                            send_msg_to_admin(&bot, &e.to_string()).await;
                        }
                    }
                }
            }
        }
    });
}
