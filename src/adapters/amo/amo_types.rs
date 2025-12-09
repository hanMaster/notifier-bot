use serde::Deserialize;

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

#[derive(Debug)]
pub struct Deal {
    pub deal_id: u64,
    pub days_limit: i32,
}
