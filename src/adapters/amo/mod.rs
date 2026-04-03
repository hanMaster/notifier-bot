use crate::adapters::amo::amo_types::{Deal, Leads};
pub(crate) use crate::adapters::amo::error::{Error, Result};
use crate::adapters::profit::ProfitbaseClient;
use crate::bot_interface::PROJECTS;
use log::info;
use reqwest::{Client, StatusCode};

mod amo_types;
mod error;

pub mod city_impl;

pub trait AmoClient {
    fn new() -> Self;
    fn base_url(&self) -> String;
    async fn get_funnel_leads(&self, funnel_id: i64) -> Result<Vec<Deal>> {
        let url = format!(
            "{}leads?filter[statuses][0][pipeline_id]={}&filter[statuses][0][status_id]={}",
            self.base_url(),
            self.pipeline_id(),
            funnel_id
        );
        info!("fetch {url}");
        let client = Client::new()
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token()));
        let response = client.send().await?;
        if response.status() == StatusCode::NO_CONTENT {
            return Ok(vec![]);
        }
        let mut data = response.json::<Leads>().await?;
        let mut next = data._links.next.take();
        let mut leads = self.extract_dkp_deals(data);

        while next.is_some() {
            let url = next.take().unwrap().href;
            info!("fetch {url}");
            let client = Client::new()
                .get(url)
                .header("Authorization", format!("Bearer {}", self.token()));

            let response = client.send().await?;

            match response.status() {
                StatusCode::OK => {
                    let mut data = response.json::<Leads>().await?;
                    next = data._links.next.take();
                    let leads_in_while = self.extract_dkp_deals(data);
                    leads.extend(leads_in_while);
                }
                StatusCode::NO_CONTENT => {
                    next = None;
                }
                status_code => {
                    return Err(Error::Funnels(format!(
                        "Fetch response status: {:?}",
                        status_code
                    )));
                }
            }
        }
        Ok(leads)
    }
    fn extract_dkp_deals(&self, leads: Leads) -> Vec<Deal>;

    fn profitbase_client(&self) -> &ProfitbaseClient;

    fn pipeline_id(&self) -> i64;
    fn token(&self) -> &str;

    fn deal_with_days_limit(&self, deal_id: u64, days: i32, project: String) -> Deal {
        let default_days_limit = if project == PROJECTS[0] { 60 } else { 30 };
        let days_limit = if days > 0 { days } else { default_days_limit };

        Deal {
            deal_id,
            days_limit,
            project,
        }
    }
}
