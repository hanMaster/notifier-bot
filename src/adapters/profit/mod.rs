pub use crate::adapters::profit::data_types::deal::DealForAdd;
use chrono::DateTime;
use data_types::{auth::AuthResponse, profit_data::ProfitRecord};
pub(crate) use error::{Error, Result};
use log::debug;
use reqwest::{Client, StatusCode};
use serde_json::json;

mod data_types;
mod error;

pub struct ProfitbaseClient {
    pub account_id: &'static str,
    pub api_key: &'static str,
}

impl ProfitbaseClient {
    pub fn new(account_id: &'static str, api_key: &'static str) -> ProfitbaseClient {
        Self {
            account_id,
            api_key,
        }
    }

    fn base_url(&self) -> String {
        format!("https://{}.profitbase.ru/api/v4/json", self.account_id)
    }

    pub async fn get_profit_token(&self) -> Result<String> {
        let payload = json!({
          "type": "api-app",
          "credentials": {
            "pb_api_key": self.api_key,
          }
        });

        let url = format!("{}/authentication", self.base_url());
        let client = Client::new().post(url).json(&payload);

        let result = client.send().await?;

        match result.status() {
            StatusCode::OK => {
                let token = result.json::<AuthResponse>().await?.access_token;
                Ok(token)
            }
            status => {
                let err = result.text().await?;
                let err_msg = format!("Failed to get token, status: {}, msg: {err}", status);
                eprintln!("{err_msg}");
                Err(Error::ProfitAuthFailed(err_msg))
            }
        }
    }

    pub async fn get_profit_data(
        &self,
        deal_id: u64,
        project: String,
        token: &str,
    ) -> Result<DealForAdd> {
        let url = format!(
            "{}/property/deal/{}?access_token={}",
            self.base_url(),
            deal_id,
            token
        );

        debug!("[get_profit_data] fetching {}", url);
        let response = Client::new()
            .get(url)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if response.status() == StatusCode::OK {
            debug!("[get_profit_data] JSON parse");
            let data = response.json::<ProfitRecord>().await?;

            debug!("[get_profit_data] received: {:?}", data);
            if data.status == "success" {
                let p = data.data.first().unwrap();

                let house_parts = p.house_name.split('№').collect::<Vec<_>>();
                let house = if house_parts.len() < 2 {
                    house_parts[0].to_string()
                } else {
                    house_parts[1].to_string()
                };
                let house = house.parse::<i32>().unwrap_or(-1);

                let sold_at = match p.sold_at.as_ref() {
                    None => {
                        match p.booked_at.as_ref() {
                            Some(booked_at) => booked_at.clone(),
                            None => {
                                let msg = format!(
                                    "Failed to parse soldAt for dealId: {deal_id}, {}, house: {house}, type: {}, № {}",
                                    project, p.property_type, p.number
                                );
                                return Err(Error::ProfitGetDataFailed(msg));
                            }
                        }
                    }
                    Some(sold_at) => {
                        sold_at.clone()
                    }
                };

                // soldAt
                let created_on = DateTime::parse_from_str(
                    format!("{} +0000", sold_at).as_str(),
                    "%Y-%m-%d %H:%M %z",
                )
                .unwrap_or(Default::default())
                .naive_local();
                let attrs = p.attributes.clone();

                Ok(DealForAdd {
                    deal_id,
                    project,
                    house,
                    object_type: p.property_type.clone(),
                    object: p.number.parse::<i32>()?,
                    facing: attrs.facing.unwrap_or("".to_string()),
                    days_limit: 30,
                    created_on,
                })
            } else {
                Err(Error::ProfitGetDataFailed(data.status))
            }
        } else {
            Err(Error::ProfitGetDataFailed(response.text().await?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::config;
    fn setup() -> ProfitbaseClient {
        let account_id = &config().PROF_CITY_ACCOUNT;
        let api_key = &config().PROF_CITY_API_KEY;
        ProfitbaseClient::new(account_id, api_key)
    }
    #[test]
    fn base_url() {
        let client = setup();
        let url = client.base_url();
        assert_eq!("https://pb18549.profitbase.ru/api/v4/json", url);
    }

    #[tokio::test]
    async fn get_profit_token() {
        let client = setup();
        let token_result = client.get_profit_token().await;
        assert!(token_result.is_ok());
        println!("{:?}", token_result.unwrap());
    }

    #[tokio::test]
    async fn get_profit_data() {
        let client = setup();
        let token = client.get_profit_token().await.unwrap();
        println!("{:?}", token);
        let data = client
            .get_profit_data(26835973, "DNS Сити".to_string(), &token)
            .await
            .unwrap();
        println!("{:?}", data);
    }
}
