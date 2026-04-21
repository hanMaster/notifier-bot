use crate::adapters::amo::AmoClient;
use crate::adapters::amo::amo_types::FlexibleType::Str;
use crate::adapters::amo::amo_types::{CustomField, Deal, Leads, Val};
use crate::adapters::profit::ProfitbaseClient;
use crate::bot_interface::PROJECTS;
use crate::config::config;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use log::info;

pub struct AmoCityClient {
    account_id: &'static str,
    token: &'static str,
    pipeline_id: i64,
    profitbase_client: ProfitbaseClient,
}

impl AmoClient for AmoCityClient {
    fn new() -> Self {
        Self {
            account_id: &config().AMO_CITY_ACCOUNT,
            token: &config().AMO_CITY_TOKEN,
            pipeline_id: 10192498,
            profitbase_client: ProfitbaseClient::new(
                &config().PROF_CITY_ACCOUNT,
                &config().PROF_CITY_API_KEY,
            ),
        }
    }

    fn base_url(&self) -> String {
        format!("https://{}.amocrm.ru/api/v4/", self.account_id)
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
                info!("================================");

                let days = l.val_to_num("Период передачи (дней)");
                info!("ID: {}, days: {}", l.id, days);

                let raw_project = l.val_to_str("ЖК");
                let project = if raw_project == PROJECTS[0] {
                    PROJECTS[0].to_string()
                } else {
                    PROJECTS[1].to_string()
                };
                info!("Project: {}", project);

                let house = l.val_to_str("Дом");
                info!("Дом: {}", house);

                let sold_at = l.val_to_str("Дата продажи для отчета");
                let ts = sold_at.parse::<i64>().unwrap_or(0);
                let created_on = ts_to_date(ts);
                info!("Sold date: {}", created_on.format("%d.%m.%Y"));

                let facing = l.val_to_str("Вид отделки квартиры");
                info!("Отделка: {}", facing);

                let property_type = l.val_to_str("Тип помещения");
                info!("Тип помещения: {}", property_type);

                let property_num = l.val_to_str("Номер помещения");
                info!("Номер помещения: {}", property_num);
                info!("================================");

                let days_limit = self.deal_days_limit(days, &project);
                Deal {
                    deal_id: l.id,
                    project,
                    house,
                    property_type,
                    property_num: property_num.parse::<i32>().unwrap_or(0),
                    facing,
                    days_limit,
                    created_on
                }
            })
            .collect::<Vec<_>>()
    }

    fn profitbase_client(&self) -> &ProfitbaseClient {
        &self.profitbase_client
    }

    fn pipeline_id(&self) -> i64 {
        self.pipeline_id
    }

    fn token(&self) -> &str {
        self.token
    }
}

pub fn ts_to_date(ts: i64) -> NaiveDateTime {
    let date = Utc.timestamp_opt(ts, 0).unwrap();
    let local_date: DateTime<Local> = DateTime::from(date);
    local_date.naive_local()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> AmoCityClient {
        AmoCityClient::new()
    }
    #[test]
    fn gen_correct_base_url() {
        let client = setup();
        let url = client.base_url();
        assert_eq!("https://dnscity.amocrm.ru/api/v4/", url);
    }

    #[tokio::test]
    async fn test_get_funnel_leads() {
        let client = setup();
        let leads = client.get_funnel_leads(65830426).await.unwrap();
        println!("{:?}", leads);
        assert_ne!(0, leads.len());
    }
}
