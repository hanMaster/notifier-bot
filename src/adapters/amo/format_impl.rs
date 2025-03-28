use crate::adapters::amo::data_types::leads::FlexibleType::Str;
use crate::adapters::amo::data_types::leads::{CustomField, Deal, Leads, Val};
use crate::adapters::amo::AmoClient;
use crate::adapters::profit::ProfitbaseClient;
use crate::bot_interface::PROJECTS;
use crate::config::config;

pub struct AmoFormatClient {
    account_id: &'static str,
    token: &'static str,
    pipeline_id: i64,
    project: &'static str,
    profitbase_client: ProfitbaseClient,
}

impl AmoClient for AmoFormatClient {
    fn new() -> Self {
        Self {
            account_id: &config().AMO_FORMAT_ACCOUNT,
            token: &config().AMO_FORMAT_TOKEN,
            pipeline_id: 1983685,
            project: PROJECTS[1],
            profitbase_client: ProfitbaseClient::new(
                &config().PROF_FORMAT_ACCOUNT,
                &config().PROF_FORMAT_API_KEY,
                PROJECTS[1],
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
                    field_id: 763071,
                    field_name: "Тип договора".to_string(),
                    values: vec![Val {
                        value: Str("ДКП".to_string()),
                        enum_id: Some(1254335),
                    }],
                })
            })
            .map(|l| {
                let flex_val = l.custom_fields_values.iter().find(|v| v.field_id == 763077);
                self.deal_with_days_limit(l.id, flex_val)
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
