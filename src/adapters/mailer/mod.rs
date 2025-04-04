use crate::adapters::mailer::data_types::{DealInfo, DkpStat, StatNumbers};
use crate::adapters::profit::DealForAdd;
use crate::config::config;
use crate::xlsx::Xlsx;
use crate::Result;
use askama::Template;
use data_types::DkpObjects;
use log::info;
use mail_send::mail_builder::MessageBuilder;
use mail_send::SmtpClientBuilder;

pub mod data_types;

pub struct Email {
    receivers: Vec<(String, String)>,
}

impl Email {
    pub fn new() -> Self {
        let receivers = Email::get_receivers();
        Self { receivers }
    }

    fn get_receivers() -> Vec<(String, String)> {
        let config = config().RECEIVERS.clone();
        let rec = config
            .split(';')
            .map(|s| {
                let parts = s.split(':').map(|p| p.to_string()).collect::<Vec<_>>();
                let name = parts.first().cloned().unwrap_or_default();
                let email = parts.last().cloned().unwrap_or_default();
                (name, email)
            })
            .collect::<Vec<(String, String)>>();
        info!("rec: {:?}", rec);
        rec
    }

    pub async fn new_objects_notification(&self, deals: Vec<DealForAdd>) -> Result<()> {
        let subject = "Новые сделки по ДКП";
        let content: Vec<DealInfo> = deals.iter().map(Into::into).collect();
        let today = chrono::Local::now().format("%d.%m.%Y %H:%M");
        let header = format!("Новые объекты по ДКП на {today}");
        let tpl = DkpObjects::new(&header, content);
        self.send(subject, tpl.render()?, None).await?;
        Ok(())
    }

    pub async fn deadline_notification(&self, deals: Vec<DealInfo>) -> Result<()> {
        let subject = "Дедлайн по передаче объектов по ДКП";
        let today = chrono::Local::now().format("%d.%m.%Y %H:%M");
        let header = format!("Дедлайн по передаче объектов на {today}");
        let tmpl = DkpObjects::new(&header, deals);
        self.send(subject, tmpl.render()?, None).await?;
        Ok(())
    }

    pub async fn stat_notification(&self, deals: Vec<DealInfo>, s: StatNumbers) -> Result<()> {
        let subject = "Статистика по объектам ДКП";
        let today = chrono::Local::now().format("%d.%m.%Y %H:%M");
        let header =
            format!("Агрегированная информация (статистика) по всем объектам ДКП на {today}");

        let tmpl = DkpStat::new(
            &header,
            s.format_apartments,
            s.format_pantries,
            s.format_parking,
            s.city_apartments,
            s.city_pantries,
        );
        let attach = Xlsx::create(deals)?;
        self.send(subject, tmpl.render()?, Some(attach)).await?;
        Ok(())
    }

    pub async fn send(
        &self,
        subject: &str,
        payload: String,
        attach: Option<Vec<u8>>,
    ) -> Result<()> {
        let username = config().LOGIN.clone();
        let secret = config().PASSWORD.clone();

        let creds = mail_send::Credentials::new(&username, &secret);

        let mut mailer = SmtpClientBuilder::new(&config().SMTP_SERVER, 587)
            .implicit_tls(false)
            .credentials(creds)
            .connect()
            .await?;

        let message = if let Some(attach) = attach {
            MessageBuilder::new()
                .from(("ДКП бот", config().FROM.as_str()))
                .to(self.receivers.clone())
                .subject(subject)
                .html_body(payload)
                .attachment(
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                    "report.xlsx",
                    attach,
                )
        } else {
            MessageBuilder::new()
                .from(("ДКП бот", config().FROM.as_str()))
                .to(self.receivers.clone())
                .subject(subject)
                .html_body(payload)
        };

        mailer.send(message).await?;
        info!("Email sent");
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use log::error;

    #[tokio::test]
    async fn mail_send_test() {
        let email = Email::new();
        let payload = "<div>Hello World</div>".to_owned();
        let subject = "Тестовое сообщение от бота";
        let send_result = email.send(&subject, payload, None).await;

        match send_result {
            Ok(_) => {
                assert!(send_result.is_ok());
            }
            Err(e) => {
                error!("Failed to connect to SMTP: {}", e)
            }
        }
    }

    #[test]
    fn test_receivers() {
        let receivers = Email::get_receivers();
        assert_ne!(receivers.len(), 0);
        println!("{:?}", receivers);
    }
}
