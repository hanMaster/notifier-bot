use crate::adapters::amo::data_types::leads::Leads;
use crate::adapters::amo::data_types::pipeline::Funnel;
pub(crate) use crate::adapters::amo::error::{Error, Result};
mod data_types;
mod error;

pub mod city_impl;
pub mod format_impl;

pub trait AmoClient{
    fn new() -> Self;
    fn base_url(&self) -> String;
    async fn get_funnels(&self) -> Result<Vec<Funnel>>;
    async fn get_funnel_leads(&self, funnel_id: i64) -> Result<Vec<u64>>;
    fn extract_lead_ids(&self, leads: Leads) -> Vec<u64>;

    fn project(&self) -> &str;
}
