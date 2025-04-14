use crate::adapters::mailer::data_types::{DealInfo, StatNumbers};
use crate::adapters::mailer::Email;
use crate::bot_interface::PROJECTS;
use crate::model::Db;
use crate::Result;

pub async fn send_stat() -> Result<()> {
    let db = Db::new().await;
    let deals = db.get_all_undone_deals().await?;
    let d_cloned = deals.clone();
    let undone_deals: Vec<DealInfo> = deals.into_iter().map(Into::into).collect();
    let format_apartments = d_cloned
        .iter()
        .filter(|d| d.project == PROJECTS[1] && d.object_type == "property")
        .count();
    let format_pantries = d_cloned
        .iter()
        .filter(|d| d.project == PROJECTS[1] && d.object_type == "pantry")
        .count();
    let format_parking = d_cloned
        .iter()
        .filter(|d| d.project == PROJECTS[1] && d.object_type == "parking")
        .count();
    let city_apartments = d_cloned
        .iter()
        .filter(|d| d.project == PROJECTS[0] && d.object_type == "property")
        .count();
    let city_pantries = d_cloned
        .iter()
        .filter(|d| d.project == PROJECTS[0] && d.object_type == "pantry")
        .count();
    let stat_numbers: StatNumbers = StatNumbers {
        format_apartments,
        format_pantries,
        format_parking,
        city_apartments,
        city_pantries,
    };

    if !undone_deals.is_empty() {
        let email = Email::new();
        email.stat_notification(undone_deals, stat_numbers).await?;
    }
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
