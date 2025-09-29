use crate::config::config;
use crate::model::deadline::search_deadline;
use crate::model::stat::send_stat;
use crate::sender::send_msg_to_admin;
use cron::Schedule;
use log::{debug, error};
use sqlx::types::chrono::Local;
use std::str::FromStr;
use teloxide::Bot;
use tokio::time::sleep;

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
                send_msg_to_admin(&bot, &info).await;

                // Stat
                let results = send_stat().await;

                if let Err(e) = results {
                    let msg = format!("Failed to send in_work stat on email: {}", e);
                    error!("{msg}");
                    send_msg_to_admin(&bot, &msg).await;
                }

                // Deadline
                let results = search_deadline().await;

                if let Err(e) = results {
                    let msg = format!("Failed to send deadline stat on email: {}", e);
                    error!("{msg}");
                    send_msg_to_admin(&bot, &msg).await;
                }
            }
        }
    });
}
