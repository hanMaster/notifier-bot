use crate::adapters::amo::AmoClient;
use crate::model::Db;
use crate::Result;
use log::{debug, error, info};

use crate::adapters::amo::city_impl::AmoCityClient;
use crate::adapters::amo::format_impl::AmoFormatClient;
use crate::adapters::mailer::Email;
use crate::adapters::profit::DealForAdd;
use crate::config::config;
use std::sync::Arc;
use teloxide::prelude::{ChatId, Requester};
use teloxide::Bot;

pub async fn sync(bot: &Bot) -> Vec<Result<Vec<DealForAdd>>> {
    let mut results: Vec<Result<Vec<DealForAdd>>> = vec![];
    let amo_city = AmoCityClient::new();
    results.extend(sync_project(amo_city, bot).await);
    let amo_format = AmoFormatClient::new();
    results.extend(sync_project(amo_format, bot).await);
    let res = notify_by_email(&results).await;
    if res.is_err() {
        error!("Failed to send email {:?}", res);
    }
    results
}

async fn notify_by_email(data: &[Result<Vec<DealForAdd>>]) -> Result<()> {
    let mut clean_data: Vec<DealForAdd> = vec![];

    data.iter().flatten().for_each(|v| {
        if !v.is_empty() {
            clean_data.extend(v.clone());
        }
    });

    if !clean_data.is_empty() {
        let email = Email::new();
        email.new_objects_notification(clean_data).await?;
    }
    Ok(())
}

async fn sync_project<A>(amo: A, bot: &Bot) -> Vec<Result<Vec<DealForAdd>>>
where
    A: AmoClient + Send + Sync + 'static,
{
    let amo = Arc::new(amo);

    let db = Db::new().await;
    let mut saved_ids = db
        .read_deal_ids_by_project(amo.project())
        .await
        .unwrap_or(vec![]);
    debug!("saved ids: {:?}", saved_ids);

    let funnels_res = amo.get_funnels().await;
    match funnels_res {
        Ok(funnels) => {
            let filtered = funnels
                .iter()
                .filter(|f| f.name.to_lowercase().contains("передача"))
                .collect::<Vec<_>>();
            let mut res = vec![];
            for funnel in filtered {
                res.push(sync_funnel(amo.clone(), &db, &mut saved_ids, funnel.id).await)
            }
            mark_as_transferred(saved_ids, bot, &db, amo.project()).await;
            res
        }
        Err(e) => {
            error!("Failed to get funnels {:?}", e);
            vec![]
        }
    }
}

async fn sync_funnel<A>(
    amo_client: Arc<A>,
    db: &Db,
    saved_ids: &mut Vec<u64>,
    funnel_id: i64,
) -> Result<Vec<DealForAdd>>
where
    A: AmoClient + Send + Sync + 'static,
{
    info!("Syncing {} funnel {}", amo_client.project(), funnel_id);
    let leads = amo_client.get_funnel_leads(funnel_id).await?;

    info!(
        "leads: {:?}",
        leads.iter().map(|l| l.deal_id).collect::<Vec<_>>()
    );

    let mut new_data: Vec<DealForAdd> = vec![];

    if !leads.is_empty() {
        let token = amo_client.profitbase_client().get_profit_token().await?;
        for lead in leads {
            if saved_ids.contains(&lead.deal_id) {
                saved_ids.retain(|i| *i != lead.deal_id);
                db.set_days_limit(amo_client.project(), lead.deal_id, lead.days_limit)
                    .await?;
                continue;
            }
            let mut profit_data = amo_client
                .profitbase_client()
                .get_profit_data(lead.deal_id, &token)
                .await?;
            profit_data.days_limit = lead.days_limit;
            db.create_deal(&profit_data).await?;
            new_data.push(profit_data);
        }
    }

    Ok(new_data)
}

async fn mark_as_transferred(remain_ids: Vec<u64>, bot: &Bot, db: &Db, project: &str) {
    if !remain_ids.is_empty() {
        info!("remain leads: {:?}", remain_ids);
        match db.mark_as_transferred(project, &remain_ids).await {
            Ok(rows) => {
                let admin_id = ChatId(config().ADMIN_ID);
                for r in rows {
                    if bot
                        .send_message(
                            admin_id,
                            format!(
                                "Проект: {}, Дом №{}, к.{} ({}) передан!",
                                r.project, r.house, r.object, r.object_type
                            ),
                        )
                        .await
                        .is_err()
                    {
                        error!("Failed to send message to admin");
                    };
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
