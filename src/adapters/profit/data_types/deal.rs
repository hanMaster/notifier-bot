use crate::model::deal::get_ru_object_type;
use chrono::NaiveDateTime;
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct DealForAdd {
    pub deal_id: u64,
    pub project: String,
    pub house: i32,
    pub object_type: String,
    pub object: i32,
    pub facing: String,
    pub days_limit: i32,
    pub created_on: NaiveDateTime,
}

impl Display for DealForAdd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let facing = if self.object_type.eq("property") {
            format!("Тип отделки: {}\n", self.facing)
        } else {
            "".to_string()
        };
        write!(
            f,
            "Проект: {}\nДом № {}\nТип объекта: {}\n№ {}\n{}Дата регистрации: {}\nПередать объект до: {}\n",
            self.project,
            self.house,
            get_ru_object_type(self.object_type.as_str()),
            self.object,
            facing,
            self.created_on.format("%d.%m.%Y"),
            self.created_on.add(Duration::from_secs(86400 * self.days_limit as u64))
                .format("%d.%m.%Y")
        )
    }
}
