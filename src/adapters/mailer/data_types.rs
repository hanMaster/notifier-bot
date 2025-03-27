use std::ops::Add;
use std::time::Duration;
use askama::Template;
use crate::adapters::profit::DealForAdd;

pub struct DealInfo {
    pub project: String,
    pub house: i32,
    pub object_type: String,
    pub object: i32,
    pub facing: String,
    pub reg_date: String,
    pub exp_date: String,
}

impl From<&DealForAdd> for DealInfo {
    fn from(value: &DealForAdd) -> Self {
        let reg_date = value.created_on.format("%d.%m.%Y").to_string();
        let exp_date = value.created_on.add(Duration::from_secs(86400 * value.days_limit as u64))
            .format("%d.%m.%Y").to_string();
        Self {
            project: value.project.clone(),
            house: value.house,
            object_type: value.object_type.clone(),
            object: value.object,
            facing: value.facing.clone(),
            reg_date,
            exp_date,
        }
    }
}

#[derive(Template)]
#[template(path = "template.html")]
pub struct DkpObjects<'a> {
    header: &'a str,
    deals: Vec<DealInfo>,
}

impl<'a> DkpObjects<'a> {
    pub fn new(header: &'a str, deals: Vec<DealInfo>) -> Self {
        Self { header, deals }
    }
}