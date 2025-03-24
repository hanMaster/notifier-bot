use crate::adapters::amo::AmoClient;
use crate::config::config;
use crate::model::Db;
use crate::Result;
use log::{error, info};

use std::sync::Arc;
use crate::adapters::profit::{DealForAdd, ProfitbaseClient};

const CITY_PIPELINE: i64 = 7486918;
const FORMAT_PIPELINE: i64 = 1983685;

pub async fn sync() -> Vec<Result<Vec<DealForAdd>>> {
    let mut results: Vec<Result<Vec<DealForAdd>>> = Vec::with_capacity(2);
    results.extend(sync_project(true).await); // city
    results.extend(sync_project(false).await); // format
    results
}

async fn sync_project(is_city: bool) -> Vec<Result<Vec<DealForAdd>>> {
    let amo = if is_city {
        Arc::new(AmoClient::new(
            &config().AMO_CITY_ACCOUNT,
            &config().AMO_CITY_TOKEN,
        ))
    } else {
        Arc::new(AmoClient::new(
            &config().AMO_FORMAT_ACCOUNT,
            &config().AMO_FORMAT_TOKEN,
        ))
    };
    let pipeline = if is_city {
        CITY_PIPELINE
    } else {
        FORMAT_PIPELINE
    };
    let funnels_res = amo.get_funnels(pipeline).await;
    match funnels_res {
        Ok(funnels) => {
            let filtered = funnels
                .iter()
                .filter(|f| f.name.to_lowercase().contains("передача"))
                .collect::<Vec<_>>();
            let mut res = vec![];
            for funnel in filtered {
                res.push(sync_funnel(is_city, amo.clone(), funnel.id).await)
            }
            res
        }
        Err(e) => {
            error!("Failed to get funnels {:?}", e);
            vec![]
        }
    }
}

async fn sync_funnel(
    is_city: bool,
    amo_client: Arc<AmoClient>,
    funnel_id: i64,
) -> Result<Vec<DealForAdd>> {
    info!(
        "Syncing {} funnel {}",
        if is_city { "city" } else { "format" },
        funnel_id
    );
    let pipeline = if is_city {
        CITY_PIPELINE
    } else {
        FORMAT_PIPELINE
    };
    let leads = amo_client.get_funnel_leads(pipeline, funnel_id).await?;

    info!("leads: {:?}", leads);

    let mut new_data: Vec<DealForAdd> = vec![];

    if !leads.is_empty() {
        let db = Db::new().await;
        let saved_ids = db.read_deal_ids().await?;
        let profit_client = if is_city {
            ProfitbaseClient::new(&config().PROF_CITY_ACCOUNT, &config().PROF_CITY_API_KEY)
        } else {
            ProfitbaseClient::new(&config().PROF_FORMAT_ACCOUNT, &config().PROF_FORMAT_API_KEY)
        };
        let token = profit_client.get_profit_token().await?;
        for lead in leads {
            if saved_ids.contains(&lead) {
                continue;
            }
            let profit_data = profit_client.get_profit_data(lead, &token).await?;
            db.create_deal(&profit_data).await?;
            new_data.push(profit_data);
        }
        db.db.close().await;
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
