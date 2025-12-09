use crate::adapters::amo::AmoClient;
use crate::model::Db;
use crate::Result;
use log::{debug, error, info};

use crate::adapters::amo::city_impl::AmoCityClient;
use crate::adapters::mailer::Email;
use crate::adapters::profit::DealForAdd;
use crate::config::config;
use crate::error::Error::AppErr;
use crate::model::deal::get_ru_object_type;
use crate::sender::send_msg_to_group;
use teloxide::Bot;

pub async fn sync(bot: &Bot) -> Result<Vec<DealForAdd>> {
    let results = sync_project(bot).await?;
    notify_by_email(&results).await?;
    Ok(results)
}

async fn notify_by_email(data: &[DealForAdd]) -> Result<()> {
    if !data.is_empty() {
        let email = Email::new();
        email.new_objects_notification(data).await?;
    }
    Ok(())
}

async fn sync_project(bot: &Bot) -> Result<Vec<DealForAdd>> {
    let db = Db::new().await;
    let mut saved_ids_limits = db.read_deal_ids().await?;
    debug!("saved ids: {:?}", saved_ids_limits);

    let res = sync_funnel(&db, &mut saved_ids_limits).await?;
    mark_as_transferred(saved_ids_limits, bot, &db).await;
    Ok(res)
}

async fn sync_funnel(
    db: &Db,
    saved_ids_limits: &mut Vec<(u64, i32, bool)>,
) -> Result<Vec<DealForAdd>> {
    let funnel_id = config().FUNNEL;
    info!("Syncing funnel {}", funnel_id);
    let amo_client = AmoCityClient::new();
    let leads = amo_client.get_funnel_leads(funnel_id).await?;

    info!(
        "leads: {:?}",
        leads.iter().map(|l| l.deal_id).collect::<Vec<_>>()
    );

    let mut new_data: Vec<DealForAdd> = vec![];

    if !leads.is_empty() {
        for lead in leads {
            let saved = saved_ids_limits
                .iter()
                .find(|i| i.0 == lead.deal_id)
                .cloned();
            if let Some(saved) = saved {
                saved_ids_limits.retain(|i| i.0 != lead.deal_id);
                // if saved days_limit not correct
                if saved.1 != lead.days_limit {
                    db.set_days_limit(&lead.project, lead.deal_id, lead.days_limit)
                        .await?;
                }
                continue;
            }

            // if deal returned to funnel we need mark it as not completed
            if db
                .mark_as_not_transferred(&lead.project, lead.deal_id)
                .await?
            {
                continue;
            }

            let token = amo_client
                .profitbase_client()
                .get_profit_token()
                .await
                .map_err(|e| AppErr(format!("Failed to get profit token {:?}", e)))?;

            let mut profit_data = amo_client
                .profitbase_client()
                .get_profit_data(lead.deal_id, lead.project, &token)
                .await
                .map_err(|e| AppErr(format!("Failed to get profit data {:?}", e)))?;

            profit_data.days_limit = lead.days_limit;
            db.create_deal(&profit_data).await?;
            new_data.push(profit_data);
        }
    }

    Ok(new_data)
}

async fn mark_as_transferred(remain_ids_limits: Vec<(u64, i32, bool)>, bot: &Bot, db: &Db) {
    if !remain_ids_limits.is_empty() {
        let remain_ids = remain_ids_limits
            .into_iter()
            .map(|(a, _, _)| a)
            .collect::<Vec<_>>();
        info!("remain leads: {:?}", remain_ids);
        match db.mark_as_transferred(&remain_ids).await {
            Ok(rows) => {
                for r in rows {
                    let msg = format!(
                        "Проект: {}, Дом №{}, к.{} ({}) передан!",
                        r.project,
                        r.house,
                        r.object,
                        get_ru_object_type(&r.object_type)
                    );
                    send_msg_to_group(bot, &msg).await;
                }
            }
            Err(e) => {
                error!("Failed to mark as transferred project: {e}");
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use sqlx::types::chrono::DateTime;
    #[test]
    fn parse_date() {
        let str_date = "2025-03-12 04:38 +0000";
        let res = DateTime::parse_from_str(str_date, "%Y-%m-%d %H:%M %z");
        println!("{:?}", res);
        assert!(res.is_ok());
    }
}
