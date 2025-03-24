use chrono::NaiveDateTime;
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DealForAdd {
    pub deal_id: u64,
    pub project: String,
    pub house: i32,
    pub object_type: String,
    pub object: i32,
    pub facing: String,
    pub created_on: NaiveDateTime,
}

impl Display for DealForAdd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let facing = if self.object_type.eq("Квартиры") {
            format!("Тип отделки: {}\n", self.facing)
        } else {
            "".to_string()
        };
        write!(
            f,
            "Проект: {}\nДом № {}\nТип объекта: {}\n№ {}\n{}Дата регистрации: {}\nПередать объект до: {}\n",
            self.project,
            self.house,
            self.object_type,
            self.object,
            facing,
            self.created_on.format("%d.%m.%Y"),
            self.created_on.add(Duration::from_secs(2592000)) // 30 days
                .format("%d.%m.%Y")
        )
    }
}
