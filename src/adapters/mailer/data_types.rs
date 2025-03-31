use crate::adapters::profit::DealForAdd;
use crate::model::deal::DealData;
use askama::Template;
use std::ops::Add;
use std::time::Duration;

#[derive(Debug, Clone)]
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
        let exp_date = value
            .created_on
            .add(Duration::from_secs(86400 * value.days_limit as u64))
            .format("%d.%m.%Y")
            .to_string();
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

impl From<DealData> for DealInfo {
    fn from(value: DealData) -> Self {
        let reg_date = value.created_on.format("%d.%m.%Y").to_string();
        let exp_date = value
            .created_on
            .add(Duration::from_secs(86400 * value.days_limit as u64))
            .format("%d.%m.%Y")
            .to_string();
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

#[derive(Template)]
#[template(path = "stat_tmpl.html")]
pub struct DkpStat<'a> {
    header: &'a str,
    format_apartments: usize,
    format_storage_rooms: usize,
    format_parking: usize,
    city_apartments: usize,
    city_storage_rooms: usize,
}

impl<'a> DkpStat<'a> {
    pub fn new(
        header: &'a str,
        format_apartments: usize,
        format_storage_rooms: usize,
        format_parking: usize,
        city_apartments: usize,
        city_storage_rooms: usize,
    ) -> Self {
        Self {
            header,
            format_apartments,
            format_storage_rooms,
            format_parking,
            city_apartments,
            city_storage_rooms,
        }
    }
}

pub struct StatNumbers {
    pub format_apartments: usize,
    pub format_storage_rooms: usize,
    pub format_parking: usize,
    pub city_apartments: usize,
    pub city_storage_rooms: usize,
}