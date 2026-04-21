use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::time::Duration;
use chrono::NaiveDateTime;
use serde::Deserialize;
use crate::model::deal::get_ru_object_type;

#[derive(Deserialize, Debug, Clone)]
pub struct Leads {
    pub _links: Links,
    pub _embedded: Embedded,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Links {
    pub next: Option<Link>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Link {
    pub href: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Embedded {
    pub leads: Vec<Lead>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct Lead {
    pub id: u64,
    pub name: String,
    pub created_at: i64,
    pub custom_fields_values: Vec<CustomField>,
}

impl Lead {
    pub fn val_to_str(&self, field_name: &str) -> String {
        let field_opt = self
            .custom_fields_values
            .iter()
            .find(|f| f.field_name == field_name);
        match field_opt {
            None => "".to_string(),
            Some(f) => f.values[0].value.clone().into(),
        }
    }
    pub fn val_to_num(&self, field_name: &str) -> i32 {
        let field_opt = self
            .custom_fields_values
            .iter()
            .find(|f| f.field_name == field_name);
        match field_opt {
            None => 0,
            Some(f) => f.values[0].value.clone().into(),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct CustomField {
    pub field_id: u64,
    pub field_name: String,
    pub values: Vec<Val>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Val {
    pub value: FlexibleType,
    pub enum_id: Option<u64>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum FlexibleType {
    Str(String),
    Int(i64),
}

impl From<FlexibleType> for i32 {
    fn from(value: FlexibleType) -> Self {
        match value {
            FlexibleType::Str(str_value) => str_value.parse().unwrap_or_default(),
            FlexibleType::Int(int_value) => int_value as i32,
        }
    }
}

impl From<FlexibleType> for String {
    fn from(value: FlexibleType) -> Self {
        match value {
            FlexibleType::Str(str_value) => str_value,
            FlexibleType::Int(val) => val.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Deal {
    pub deal_id: u64,
    pub project: String,
    pub house: String,
    pub property_type: String,
    pub property_num: i32,
    pub facing: String,
    pub days_limit: i32,
    pub created_on: NaiveDateTime,
}

impl Display for Deal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let facing = if self.property_type.eq("property") {
            format!("Тип отделки: {}\n", self.facing)
        } else {
            "".to_string()
        };
        write!(
            f,
            "Сделка: {}\nПроект: {}\n{}\n{} № {}\n{}Дата регистрации: {}\nПередать объект до: {}\n",
            self.deal_id,
            self.project,
            self.house,
            get_ru_object_type(self.property_type.as_str()),
            self.property_num,
            facing,
            self.created_on.format("%d.%m.%Y"),
            self.created_on
                .add(Duration::from_secs(86400 * self.days_limit as u64))
                .format("%d.%m.%Y")
        )
    }
}