use crate::adapters::profit::DealForAdd;
use crate::model::deal::{DealData, get_ru_object_type};
use askama::Template;
use chrono::NaiveDateTime;
use std::ops::Add;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DealInfo {
    pub project: String,
    pub house: String,
    pub property_type: String,
    pub property_num: i32,
    pub facing: String,
    pub reg_date: String,
    pub exp_date: String,
}

impl DealInfo {
    fn from_deal(
        created_on: &NaiveDateTime,
        days_limit: u64,
        project: &str,
        house: &str,
        property_type: &str,
        property_num: i32,
        facing: &str,
    ) -> Self {
        let reg_date = created_on.format("%d.%m.%Y").to_string();
        let exp_date = created_on
            .add(Duration::from_secs(86400 * days_limit))
            .format("%d.%m.%Y")
            .to_string();
        Self {
            project: project.to_string(),
            house: house.to_string(),
            property_type: get_ru_object_type(property_type).to_string(),
            property_num,
            facing: facing.to_string(),
            reg_date,
            exp_date,
        }
    }
}

impl From<&DealForAdd> for DealInfo {
    fn from(d: &DealForAdd) -> Self {
        DealInfo::from_deal(
            &d.created_on,
            d.days_limit as u64,
            &d.project,
            &d.house,
            &d.property_type,
            d.property_num,
            &d.facing,
        )
    }
}

impl From<DealData> for DealInfo {
    fn from(d: DealData) -> Self {
        DealInfo::from_deal(
            &d.created_on,
            d.days_limit as u64,
            &d.project,
            &d.house,
            &d.property_type,
            d.property_num,
            &d.facing,
        )
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

#[derive(Template)]
#[template(path = "stat_tmpl.html")]
pub struct DkpStat<'a> {
    header: &'a str,
    format_apartments: usize,
    format_pantries: usize,
    format_parking: usize,
    city_apartments: usize,
    city_pantries: usize,
}

impl<'a> DkpStat<'a> {
    pub fn new(
        header: &'a str,
        format_apartments: usize,
        format_pantries: usize,
        format_parking: usize,
        city_apartments: usize,
        city_pantries: usize,
    ) -> Self {
        Self {
            header,
            format_apartments,
            format_pantries,
            format_parking,
            city_apartments,
            city_pantries,
        }
    }
}

pub struct StatNumbers {
    pub format_apartments: usize,
    pub format_pantries: usize,
    pub format_parking: usize,
    pub city_apartments: usize,
    pub city_pantries: usize,
}
