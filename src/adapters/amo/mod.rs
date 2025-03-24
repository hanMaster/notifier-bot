use crate::adapters::amo::data_types::leads::FlexibleType::Str;
use crate::adapters::amo::data_types::leads::{CustomField, Leads, Val};
use crate::adapters::amo::data_types::pipeline::{Funnel, Pipeline};
pub(crate) use crate::adapters::amo::error::{Error, Result};
use reqwest::{Client, StatusCode};
mod data_types;
mod error;

pub struct AmoClient {
    account_id: &'static str,
    token: &'static str,
}

impl AmoClient {
    pub fn new(account_id: &'static str, token: &'static str) -> Self {
        AmoClient { account_id, token }
    }

    fn base_url(&self) -> String {
        format!("https://{}.amocrm.ru/api/v4/", self.account_id)
    }

    pub async fn get_funnels(&self, pipeline_id: i64) -> Result<Vec<Funnel>> {
        let url = format!("{}leads/pipelines/{}", self.base_url(), pipeline_id);
        let client = reqwest::Client::new()
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token));
        let response = client.send().await?;
        match response.status() {
            StatusCode::OK => {
                let data = response.json::<Pipeline>().await?;
                let funnels = data._embedded.statuses;
                Ok(funnels)
            }
            _ => {
                let body = response.text().await?;
                eprintln!("Failed to get funnels: {}", body);
                Err(Error::Funnels(body))
            }
        }
    }

    pub async fn get_funnel_leads(&self, pipeline_id: i64, funnel_id: i64) -> Result<Vec<u64>> {
        let url = format!(
            "{}leads?filter[statuses][0][pipeline_id]={}&filter[statuses][0][status_id]={}",
            self.base_url(),
            pipeline_id,
            funnel_id
        );
        let client = Client::new()
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token));
        let response = client.send().await?;
        if response.status() == StatusCode::NO_CONTENT {
            return Ok(vec![]);
        }
        let mut data = response.json::<Leads>().await?;
        let mut next = data._links.next.take();
        let mut leads = self.extract_lead_ids(data);

        while next.is_some() {
            let client = Client::new()
                .get(next.as_ref().unwrap().href.to_string())
                .header("Authorization", format!("Bearer {}", self.token));
            let mut data = client.send().await?.json::<Leads>().await?;

            next = data._links.next.take();
            let leads_in_while = self.extract_lead_ids(data);

            leads.extend(leads_in_while);
        }
        Ok(leads)
    }

    fn extract_lead_ids(&self, leads: Leads) -> Vec<u64> {
        leads
            ._embedded
            .leads
            .iter()
            .filter(|l| {
                l.custom_fields_values.contains(&CustomField {
                    field_id: 1631153,
                    field_name: "Тип договора".to_string(),
                    values: vec![Val {
                        value: Str("ДКП".to_string()),
                        enum_id: Some(4661181),
                    }],
                })
            })
            .map(|l| l.id)
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::config;

    fn setup() -> AmoClient {
        AmoClient::new(&config().AMO_CITY_ACCOUNT, &config().AMO_CITY_TOKEN)
    }
    #[test]
    fn gen_correct_base_url() {
        let client = setup();
        let url = client.base_url();
        assert_eq!("https://dnscity.amocrm.ru/api/v4/", url);
    }

    #[tokio::test]
    async fn test_get_funnels() {
        let client = setup();
        let funnels = client.get_funnels(7486918).await.unwrap();
        assert_ne!(0, funnels.len());
    }

    #[tokio::test]
    async fn test_get_funnel_leads() {
        let client = setup();
        let leads = client.get_funnel_leads(7486918, 65830426).await.unwrap();
        println!("{:?}", leads);
        assert_ne!(0, leads.len());
    }
}
