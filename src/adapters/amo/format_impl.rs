use crate::adapters::amo::data_types::leads::FlexibleType::Str;
use crate::adapters::amo::data_types::leads::{CustomField, Leads, Val};
use crate::adapters::amo::data_types::pipeline::{Funnel, Pipeline};
use crate::adapters::amo::{AmoClient, Error};
use crate::config::config;
use reqwest::{Client, StatusCode};

pub struct AmoFormatClient {
    account_id: &'static str,
    token: &'static str,
    pipeline_id: i64,
    project: &'static str,
}

impl AmoClient for AmoFormatClient {
    fn new() -> Self {
        Self {
            account_id: &config().AMO_FORMAT_ACCOUNT,
            token: &config().AMO_FORMAT_TOKEN,
            pipeline_id: 1983685,
            project: "ЖК Формат",
        }
    }

    fn base_url(&self) -> String {
        format!("https://{}.amocrm.ru/api/v4/", self.account_id)
    }

    async fn get_funnels(&self) -> crate::adapters::amo::Result<Vec<Funnel>> {
        let url = format!("{}leads/pipelines/{}", self.base_url(), self.pipeline_id);
        let client = Client::new()
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

    async fn get_funnel_leads(&self, funnel_id: i64) -> crate::adapters::amo::Result<Vec<u64>> {
        let url = format!(
            "{}leads?filter[statuses][0][pipeline_id]={}&filter[statuses][0][status_id]={}",
            self.base_url(),
            self.pipeline_id,
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
                    field_id: 763071,
                    field_name: "Тип договора".to_string(),
                    values: vec![Val {
                        value: Str("ДКП".to_string()),
                        enum_id: Some(1254335),
                    }],
                })
            })
            .map(|l| l.id)
            .collect::<Vec<_>>()
    }

    fn project(&self) -> &str {
        self.project
    }
}
