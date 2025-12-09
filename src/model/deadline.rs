use crate::Result;
use crate::adapters::mailer::Email;
use crate::adapters::mailer::data_types::DealInfo;
use crate::model::Db;
use chrono::{Local, TimeZone};
use log::{debug, info};
use std::ops::Add;
use std::time::Duration;

pub async fn search_deadline() -> Result<()> {
    info!("Searching for deadline objects");
    let db = Db::new().await;
    let deals = db.get_all_undone_deals().await?;
    let mut deadline_objects: Vec<DealInfo> = vec![];
    let now = Local::now();
    for deal in deals {
        let exp_date = deal
            .created_on
            .add(Duration::from_secs(86400 * deal.days_limit as u64));

        let exp_dt = Local.from_local_datetime(&exp_date).unwrap();

        let delta = exp_dt - now;

        if delta.num_days() < 5 {
            deadline_objects.push(deal.into());
        }
    }
    debug!("{:#?}", deadline_objects);
    debug!("Found {:#?} deadlines", deadline_objects.len());

    if !deadline_objects.is_empty() {
        let email = Email::new();
        email.deadline_notification(deadline_objects).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_search_deadline() {
        let res = search_deadline().await;
        if res.is_err() {
            println!("Error: {:?}", res);
        }
        assert!(res.is_ok());
    }
}
