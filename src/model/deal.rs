use crate::adapters::profit::DealForAdd;
use crate::model::Db;
use crate::Result;
use log::{debug, error, info};
use sqlx::types::chrono::NaiveDateTime;
use sqlx::FromRow;
use std::ops::Add;
use std::time::Duration;

#[allow(dead_code)]
#[derive(FromRow, Clone)]
pub struct DealData {
    pub id: i32,
    pub deal_id: u64,
    pub project: String,
    pub house: i32,
    pub object_type: String,
    pub object: i32,
    pub facing: String,
    pub days_limit: i32,
    pub transfer_completed: bool,
    pub created_on: NaiveDateTime,
    pub updated_on: String,
}
#[derive(FromRow, Debug)]
pub struct HouseNumbers {
    pub house: i32,
}
#[derive(FromRow, Debug)]
pub struct ObjectNumbers {
    pub object: i32,
}

impl Db {
    pub async fn get_all_undone_deals(&self) -> Result<Vec<DealData>> {
        let records: Vec<DealData> =
            sqlx::query_as("SELECT * FROM deal WHERE transfer_completed = false")
                .fetch_all(&self.db)
                .await?;
        debug!("[list_objects] Records fetched {}", records.len());
        Ok(records)
    }

    pub async fn list_house_numbers(&self, project: &str, object_type: &str) -> Result<Vec<i32>> {
        let records: Vec<HouseNumbers> = sqlx::query_as(
            r#"SELECT DISTINCT house
                    FROM deal
                    WHERE project = $1
                      AND object_type = $2
                      AND transfer_completed = false
                    ORDER BY house "#,
        )
        .bind(project)
        .bind(object_type)
        .fetch_all(&self.db)
        .await?;
        debug!("[list_house_numbers] {:#?}", records);
        let res = records.iter().map(|r| r.house).collect();
        Ok(res)
    }

    pub async fn list_numbers(
        &self,
        project: &str,
        object_type: &str,
        house: i32,
    ) -> Result<Vec<i32>> {
        let records: Vec<ObjectNumbers> = sqlx::query_as(
            r#"SELECT object
            FROM deal
            WHERE project = $1
              AND object_type = $2
              AND house = $3
              AND transfer_completed = false
            ORDER BY object "#,
        )
        .bind(project)
        .bind(object_type)
        .bind(house)
        .fetch_all(&self.db)
        .await?;
        let res = records.iter().map(|r| r.object).collect();
        Ok(res)
    }

    pub async fn create_deal(&self, d: &DealForAdd) -> Result<()> {
        debug!("create deal with data: {:?}", &d);
        let (id, ): (i64,) = sqlx::query_as(
            r#"
                INSERT INTO deal (deal_id, project, house, object_type, object, facing, days_limit, created_on)
                VALUES($1, $2, $3, $4, $5, $6, $7, $8) returning id"#,
        )
            .bind(d.deal_id as i64)
            .bind(&d.project)
            .bind(d.house)
            .bind(&d.object_type)
            .bind(d.object)
            .bind(&d.facing)
            .bind(d.days_limit)
            .bind(d.created_on)
            .fetch_one(&self.db)
            .await?;
        debug!("Created row with id: {}", id);
        Ok(())
    }

    pub async fn mark_as_transferred(&self, ids: &[u64]) -> Result<Vec<DealData>> {
        info!("mark as transferred ids: {:?}", ids);
        for id in ids {
            let res = sqlx::query(
                r#"
                UPDATE deal SET transfer_completed = true
                            WHERE deal.deal_id = $1"#,
            )
            .bind(*id as i64)
            .execute(&self.db)
            .await?;
            debug!("{:?}", res);
        }

        let ids_str = ids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            "SELECT * FROM deal WHERE transfer_completed = true AND deal_id in ({ids_str})"
        );

        let done_objects: Vec<DealData> = sqlx::query_as(&query).fetch_all(&self.db).await?;

        Ok(done_objects)
    }

    pub async fn mark_as_not_transferred(&self, project: &str, deal_id: u64) -> Result<bool> {
        let res = sqlx::query(
            r#"
                UPDATE deal SET transfer_completed = false
                            WHERE project = $1 AND deal.deal_id = $2"#,
        )
        .bind(project)
        .bind(deal_id as i64)
        .execute(&self.db)
        .await?;

        let updated = res.rows_affected() > 0;

        if updated {
            info!("mark as not transferred project: {project}, deal_id: {deal_id}");
        }

        Ok(updated)
    }

    pub async fn set_days_limit(&self, project: &str, deal_id: u64, days_limit: i32) -> Result<()> {
        info!("[set_days_limit] project: {project}, deal_id: {deal_id}, limit: {days_limit}");
        let res = sqlx::query(
            r#"
                UPDATE deal SET days_limit = $1
                            WHERE project = $2 AND deal_id = $3"#,
        )
        .bind(days_limit)
        .bind(project)
        .bind(deal_id as i64)
        .execute(&self.db)
        .await?;
        info!("[set_days_limit] update result: {:?}", res);
        Ok(())
    }

    pub async fn read_deal_ids(&self) -> Result<Vec<(u64, i32, bool)>> {
        let records: Vec<DealData> =
            sqlx::query_as("SELECT * FROM deal WHERE transfer_completed = false")
                .fetch_all(&self.db)
                .await?;
        let res = records
            .iter()
            .map(|r| (r.deal_id, r.days_limit, r.transfer_completed))
            .collect();
        Ok(res)
    }

    async fn get_deal(
        &self,
        project: &str,
        object_type: &str,
        house: i32,
        number: i32,
    ) -> Result<DealData> {
        let rows = sqlx::query_as(
            r#"
            SELECT * FROM deal
                     WHERE project = $1
                       AND object_type = $2
                       AND house = $3
                       AND object = $4 "#,
        )
        .bind(project)
        .bind(object_type)
        .bind(house)
        .bind(number)
        .fetch_one(&self.db)
        .await?;
        Ok(rows)
    }
}

pub async fn get_house_numbers(project: &str, object_type: &str) -> Vec<i32> {
    let db = Db::new().await;
    let profit_type = get_en_object_type(object_type);
    let res = db.list_house_numbers(project, profit_type).await;
    res.unwrap_or_else(|e| {
        error!("[get_house_numbers] {:?}", e);
        vec![]
    })
}

pub async fn get_object_numbers(project: &str, object_type: &str, house: i32) -> Vec<i32> {
    let db = Db::new().await;
    let profit_type = get_en_object_type(object_type);
    let res = db.list_numbers(project, profit_type, house).await;
    res.unwrap_or_else(|e| {
        error!("[get_object_numbers] {:?}", e);
        vec![]
    })
}

pub fn get_ru_object_type(profitbase_type: &str) -> &'static str {
    match profitbase_type {
        "property" => "Квартира",
        "pantry" => "Кладовка",
        "parking" => "Машиноместо",
        _ => "",
    }
}

pub fn get_en_object_type(object_type: &str) -> &'static str {
    match object_type {
        "Квартиры" => "property",
        "Кладовки" => "pantry",
        "Машиноместа" => "parking",
        _ => "",
    }
}

pub async fn prepare_response(project: &str, object_type: &str, house: i32, number: i32) -> String {
    let db = Db::new().await;
    let profit_type = get_en_object_type(object_type);
    let result = db.get_deal(project, profit_type, house, number).await;

    match result {
        Ok(b) => {
            let facing = if b.object_type.eq("property") {
                format!("Тип отделки: {}\n", b.facing)
            } else {
                "".to_string()
            };

            let res = format!(
                "Проект: {}\nДом № {}\nТип объекта: {}\n№ {}\n{}Дата регистрации: {}\nПередать объект до: {}\n",
                b.project,
                b.house,
                get_ru_object_type(b.object_type.as_str()),
                b.object,
                facing,
                b.created_on.format("%d.%m.%Y"),
                b.created_on
                    .add(Duration::from_secs(86400 * b.days_limit as u64))
                    .format("%d.%m.%Y")
            );

            if res.is_empty() {
                "Нет данных".to_string()
            } else {
                res
            }
        }

        Err(e) => {
            error!("Prepare response error: {}", e);
            "Ошибка чтения данных".to_string()
        }
    }
}
