use crate::Result;
use crate::adapters::mailer::Email;
use crate::adapters::mailer::data_types::{DealInfo, StatNumbers};
use crate::bot_interface::PROJECTS;
use crate::model::Db;

pub async fn send_stat() -> Result<()> {
    let db = Db::new().await;
    let deals_in_work = db.get_all_undone_deals().await?;

    if deals_in_work.is_empty() {
        return Ok(());
    }

    let format_apartments = deals_in_work
        .iter()
        .filter(|d| d.project == PROJECTS[1] && d.object_type == "property")
        .count();
    let format_pantries = deals_in_work
        .iter()
        .filter(|d| d.project == PROJECTS[1] && d.object_type == "pantry")
        .count();
    let format_parking = deals_in_work
        .iter()
        .filter(|d| d.project == PROJECTS[1] && d.object_type == "parking")
        .count();
    let city_apartments = deals_in_work
        .iter()
        .filter(|d| d.project == PROJECTS[0] && d.object_type == "property")
        .count();
    let city_pantries = deals_in_work
        .iter()
        .filter(|d| d.project == PROJECTS[0] && d.object_type == "pantry")
        .count();
    let stat_numbers = StatNumbers {
        format_apartments,
        format_pantries,
        format_parking,
        city_apartments,
        city_pantries,
    };
    let undone_deals: Vec<DealInfo> = deals_in_work.into_iter().map(Into::into).collect();

    let email = Email::new();
    email.stat_notification(undone_deals, stat_numbers).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_send_stat() {
        let res = send_stat().await;
        if res.is_err() {
            println!("Error: {:?}", res);
        }
        assert!(res.is_ok());
    }
}
