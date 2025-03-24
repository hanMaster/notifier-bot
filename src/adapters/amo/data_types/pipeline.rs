use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Pipeline {
    pub _embedded: Embedded,
}

#[derive(Debug, Deserialize)]
pub struct Embedded {
    pub statuses: Vec<Funnel>,
}

#[derive(Debug, Deserialize)]
pub struct Funnel {
    pub id: i64,
    pub name: String,
}
