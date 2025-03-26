use crate::adapters::amo::data_types::leads::FlexibleType::Str;
use crate::adapters::amo::data_types::leads::{CustomField, Deal, Leads, Val};
use crate::adapters::amo::AmoClient;
use crate::adapters::profit::ProfitbaseClient;
use crate::bot_interface::PROJECTS;
use crate::config::config;
use log::debug;

pub struct AmoCityClient {
    account_id: &'static str,
    token: &'static str,
    pipeline_id: i64,
    project: &'static str,
    profitbase_client: ProfitbaseClient,
}

impl AmoClient for AmoCityClient {
    fn new() -> Self {
        Self {
            account_id: &config().AMO_CITY_ACCOUNT,
            token: &config().AMO_CITY_TOKEN,
            pipeline_id: 7486918,
            project: PROJECTS[0],
            profitbase_client: ProfitbaseClient::new(
                &config().PROF_CITY_ACCOUNT,
                &config().PROF_CITY_API_KEY,
                PROJECTS[0],
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
                let id = l.id;
                let flex_val = l
                    .custom_fields_values
                    .iter()
                    .find(|v| v.field_id == 1635059);

                debug!("FLEX {:?}", flex_val);

                if let Some(custom_field) = flex_val {
                    let flex_val = custom_field.values.first().unwrap().clone();
                    let days_limit = if let Str(val) = flex_val.value {
                        val.parse::<i32>().unwrap_or(30)
                    } else {
                        30
                    };
                    Deal {
                        deal_id: id,
                        days_limit,
                    }
                } else {
                    Deal {
                        deal_id: id,
                        days_limit: 30,
                    }
                }
            })
            .collect::<Vec<_>>()
    }

    fn project(&self) -> &str {
        self.project
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
    async fn test_get_funnels() {
        let client = setup();
        let funnels = client.get_funnels().await.unwrap();
        assert_ne!(0, funnels.len());
    }

    #[tokio::test]
    async fn test_get_funnel_leads() {
        let client = setup();
        let leads = client.get_funnel_leads(65830426).await.unwrap();
        println!("{:?}", leads);
        assert_ne!(0, leads.len());
    }
}
