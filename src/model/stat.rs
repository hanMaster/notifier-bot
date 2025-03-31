use crate::adapters::mailer::data_types::DealInfo;
use crate::adapters::mailer::Email;
use crate::model::Db;
use crate::Result;

pub async fn send_stat() -> Result<()> {
    let db = Db::new().await;
    let deals = db.get_all_undone_deals().await?;
    let undone_deals: Vec<DealInfo> = deals.into_iter().map(|d| d.into()).collect();

    if !undone_deals.is_empty() {
        let email = Email::new();
        email.stat_notification(undone_deals).await?;
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
