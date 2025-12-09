use crate::adapters::amo::amo_types::{CustomField, Deal, Leads};
pub(crate) use crate::adapters::amo::error::{Error, Result};
use crate::adapters::profit::ProfitbaseClient;
use crate::bot_interface::PROJECTS;
use log::debug;
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
            let client = Client::new()
                .get(next.take().unwrap().href)
                .header("Authorization", format!("Bearer {}", self.token()));
            let mut data = client.send().await?.json::<Leads>().await?;

            next = data._links.next.take();
            let leads_in_while = self.extract_dkp_deals(data);

            leads.extend(leads_in_while);
        }
        Ok(leads)
    }
    fn extract_dkp_deals(&self, leads: Leads) -> Vec<Deal>;

    fn project(&self) -> &str;

    fn profitbase_client(&self) -> &ProfitbaseClient;

    fn pipeline_id(&self) -> i64;
    fn token(&self) -> &str;

    fn deal_with_days_limit(&self, deal_id: u64, flex_value: Option<&CustomField>) -> Deal {
        let default_days_limit = if self.project().eq(PROJECTS[0]) {
            60
        } else {
            30
        };
        match flex_value {
            None => Deal {
                deal_id,
                days_limit: default_days_limit,
            },
            Some(custom_field) => {
                let flex_val = custom_field.values.first().unwrap().clone();
                let days_limit = flex_val.value.into();
                debug!("PARSED days_limit: {:?}", days_limit);

                Deal {
                    deal_id,
                    days_limit,
                }
            }
        }
    }
}
