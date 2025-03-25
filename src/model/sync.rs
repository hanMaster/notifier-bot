use crate::adapters::amo::AmoClient;
use crate::model::Db;
use crate::Result;
use log::{error, info};

use crate::adapters::amo::city_impl::AmoCityClient;
use crate::adapters::amo::format_impl::AmoFormatClient;
use crate::adapters::profit::DealForAdd;
use std::sync::Arc;

pub async fn sync() -> Vec<Result<Vec<DealForAdd>>> {
    let mut results: Vec<Result<Vec<DealForAdd>>> = vec![];
    let amo_city = AmoCityClient::new();
    results.extend(sync_project(amo_city).await);
    let amo_format = AmoFormatClient::new();
    results.extend(sync_project(amo_format).await);
    results
}

async fn sync_project<A>(amo: A) -> Vec<Result<Vec<DealForAdd>>>
where
    A: AmoClient + Send + Sync + 'static,
{
    let amo = Arc::new(amo);

    let db = Db::new().await;
    let mut saved_ids = db.read_deal_ids_by_project(amo.project()).await.unwrap_or(vec![]);
    println!("saved ids: {:?}", saved_ids);

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
            println!("remain leads: {:?}", saved_ids);
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

    info!("leads: {:?}", leads);

    let mut new_data: Vec<DealForAdd> = vec![];

    if !leads.is_empty() {
        let token = amo_client.profitbase_client().get_profit_token().await?;
        for lead in leads {
            if saved_ids.contains(&lead) {
                saved_ids.retain(|i| *i != lead);
                continue;
            }
            let profit_data = amo_client
                .profitbase_client()
                .get_profit_data(lead, &token)
                .await?;
            db.create_deal(&profit_data).await?;
            new_data.push(profit_data);
        }
    }

    Ok(new_data)
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
