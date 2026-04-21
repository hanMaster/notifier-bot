use crate::model::deal::get_ru_object_type;
use chrono::NaiveDateTime;
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct DealForAdd {
    pub deal_id: u64,
    pub project: String,
    pub house: String,
    pub property_type: String,
    pub property_num: i32,
    pub facing: String,
    pub days_limit: i32,
    pub created_on: NaiveDateTime,
}

