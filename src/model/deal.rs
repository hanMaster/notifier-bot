use crate::Result;
use crate::adapters::amo::amo_types::Deal;
use crate::model::Db;
use log::{debug, error, info};
use sqlx::FromRow;
use sqlx::types::chrono::NaiveDateTime;
use std::ops::Add;
use std::time::Duration;

#[allow(dead_code)]
#[derive(FromRow, Clone)]
pub struct DealData {
    pub id: i32,
    pub deal_id: u64,
    pub project: String,
    pub house: String,
    pub property_type: String,
    pub property_num: i32,
    pub facing: String,
    pub days_limit: i32,
    pub transfer_completed: bool,
    pub created_on: NaiveDateTime,
    pub updated_on: String,
}
#[derive(FromRow, Debug)]
pub struct HouseNumbers {
    pub house: String,
}
#[derive(FromRow, Debug)]
pub struct PropertyNumbers {
    pub property_num: i32,
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

    pub async fn list_house_numbers(
        &self,
        project: &str,
        property_type: &str,
    ) -> Result<Vec<String>> {
        let records: Vec<HouseNumbers> = sqlx::query_as(
            r#"SELECT DISTINCT house
                    FROM deal
                    WHERE project = $1
                      AND property_type = $2
                      AND transfer_completed = false
                    ORDER BY house "#,
        )
        .bind(project)
        .bind(property_type)
        .fetch_all(&self.db)
        .await?;
        debug!("[list_house_numbers] {:#?}", records);
        let res = records.into_iter().map(|r| r.house).collect();
        Ok(res)
    }

    pub async fn list_numbers(
        &self,
        project: &str,
        property_type: &str,
        house: &str,
    ) -> Result<Vec<i32>> {
        let records: Vec<PropertyNumbers> = sqlx::query_as(
            r#"SELECT property_num
            FROM deal
            WHERE project = $1
              AND property_type = $2
              AND house = $3
              AND transfer_completed = false
            ORDER BY property_num "#,
        )
        .bind(project)
        .bind(property_type)
        .bind(house)
        .fetch_all(&self.db)
        .await?;
        let res = records.iter().map(|r| r.property_num).collect();
        Ok(res)
    }

    pub async fn create_deal(&self, d: &Deal) -> Result<()> {
        debug!("create deal with data: {:?}", &d);
        let (id, ): (i64,) = sqlx::query_as(
            r#"
                INSERT INTO deal (deal_id, project, house, property_type, property_num, facing, days_limit, created_on)
                VALUES($1, $2, $3, $4, $5, $6, $7, $8) returning id"#,
        )
            .bind(d.deal_id as i64)
            .bind(&d.project)
            .bind(&d.house)
            .bind(&d.property_type)
            .bind(d.property_num)
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
        house: &str,
        number: i32,
    ) -> Result<DealData> {
        let rows = sqlx::query_as(
            r#"
            SELECT * FROM deal
                     WHERE project = $1
                       AND property_type = $2
                       AND house = $3
                       AND property_num = $4 "#,
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

pub async fn get_house_numbers(project: &str, property_type: &str) -> Vec<String> {
    let db = Db::new().await;
    let res = db.list_house_numbers(project, property_type).await;
    res.unwrap_or_else(|e| {
        error!("[get_house_numbers] {:?}", e);
        vec![]
    })
}

pub async fn get_property_numbers(project: &str, property_type: &str, house: &str) -> Vec<i32> {
    let db = Db::new().await;
    let res = db.list_numbers(project, property_type, house).await;
    res.unwrap_or_else(|e| {
        error!("[get_object_numbers] {:?}", e);
        vec![]
    })
}

pub async fn prepare_response(
    project: &str,
    property_type: &str,
    house: &str,
    number: i32,
) -> String {
    let db = Db::new().await;
    let result = db.get_deal(project, property_type, house, number).await;

    match result {
        Ok(b) => {
            let facing = if b.property_type.eq("property") {
                format!("Тип отделки: {}\n", b.facing)
            } else {
                "".to_string()
            };

            let res = format!(
                "Сделка: {}\nПроект: {}\n{}\nТип объекта: {}\n№ {}\n{}Дата регистрации: {}\nПередать объект до: {}\n",
                b.deal_id,
                b.project,
                b.house,
                b.property_type,
                b.property_num,
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
