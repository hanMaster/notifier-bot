use crate::adapters::amo::amo_types::FlexibleType::Str;
use crate::adapters::amo::amo_types::{CustomField, Deal, Leads, Val};
pub(crate) use crate::adapters::amo::error::{Error, Result};
use crate::bot_interface::PROJECTS;
use crate::config::config;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use log::{debug, info};
use reqwest::{Client, StatusCode};

pub mod amo_types;
mod error;

pub struct AmoClient {
    account_id: &'static str,
    token: &'static str,
    pipeline_id: i64,
}

impl AmoClient {
    pub(crate) fn new() -> Self {
        Self {
            account_id: &config().AMO_CITY_ACCOUNT,
            token: &config().AMO_CITY_TOKEN,
            pipeline_id: 10192498,
        }
    }
    fn base_url(&self) -> String {
        format!("https://{}.amocrm.ru/api/v4/", self.account_id)
    }
    pub(crate) async fn get_funnel_leads(&self, funnel_id: i64) -> Result<Vec<Deal>> {
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
    fn extract_dkp_deals(&self, leads: Leads) -> Vec<Deal> {
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
            .map(|l| {
                debug!("================================");

                let days = l.val_to_num("Период передачи (дней)");
                debug!("ID: {}, days: {}", l.id, days);

                let raw_project = l.val_to_str("ЖК");
                let project = if raw_project == PROJECTS[0] {
                    PROJECTS[0].to_string()
                } else {
                    PROJECTS[1].to_string()
                };
                debug!("Project: {}", project);

                let house = l.val_to_str("Дом");
                debug!("Дом: {}", house);

                let sold_at = l.val_to_str("Дата продажи для отчета");
                let ts = sold_at.parse::<i64>().unwrap_or(0);
                let created_on = ts_to_date(ts);
                debug!("Sold date: {}", created_on.format("%d.%m.%Y"));

                let facing = l.val_to_str("Вид отделки квартиры");
                debug!("Отделка: {}", facing);

                let property_type = l.val_to_str("Тип помещения");
                debug!("Тип помещения: {}", property_type);

                let property_num = l.val_to_str("Номер помещения");
                debug!("Номер помещения: {}", property_num);
                debug!("================================");

                let days_limit = self.deal_days_limit(days, &project);
                Deal {
                    deal_id: l.id,
                    project,
                    house,
                    property_type,
                    property_num: property_num.parse::<i32>().unwrap_or(0),
                    facing,
                    days_limit,
                    created_on,
                }
            })
            .collect::<Vec<_>>()
    }

    fn pipeline_id(&self) -> i64 {
        self.pipeline_id
    }
    fn token(&self) -> &str {
        self.token
    }

    fn deal_days_limit(&self, days: i32, project: &str) -> i32 {
        let default_days_limit = if project == PROJECTS[0] { 60 } else { 30 };
        if days > 0 { days } else { default_days_limit }
    }
}

pub fn ts_to_date(ts: i64) -> NaiveDateTime {
    let date = Utc.timestamp_opt(ts, 0).unwrap();
    let local_date: DateTime<Local> = DateTime::from(date);
    local_date.naive_local()
}
