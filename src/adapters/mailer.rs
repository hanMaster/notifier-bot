use crate::adapters::profit::DealForAdd;
use crate::config::config;
use crate::Result;
use askama::Template;
use log::info;
use mail_send::mail_builder::MessageBuilder;
use mail_send::SmtpClientBuilder;
use std::ops::Add;
use std::time::Duration;

#[derive(Template)]
#[template(path = "new_objects.html")]
struct NewObjects<'a> {
    date: &'a str,
    payload: &'a str,
}

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
                let parts = s.split(':').collect::<Vec<_>>();
                (parts[0].to_string(), parts[1].to_string())
            })
            .collect::<Vec<(String, String)>>();
        info!("rec: {:?}", rec);
        rec
    }

    pub async fn new_objects_notification(&self, deals: Vec<DealForAdd>) -> Result<()> {
        let subject = "Новые сделки по ДКП";
        // let content = deals.iter().map(|d| d.to_string()).collect::<Vec<_>>();
        // let rows = content.join("<br />");
        let facing = if deals[0].object_type.eq("Квартиры") {
            format!("Тип отделки: {}\n", deals[0].facing)
        } else {
            "".to_string()
        };
        let payload = format!(
            "<p>Проект: {}<br/>
            Дом № {}<br/>
            Тип объекта: {}<br/>
            № {}<br/>
            Тип отделки: {}<br/>
            Дата регистрации: {}<br/>
            Передать объект до: {}</p><br/>",
            deals[0].project,
            deals[0].house,
            deals[0].object_type,
            deals[0].object,
            facing,
            deals[0].created_on.format("%d.%m.%Y"),
            deals[0]
                .created_on
                .add(Duration::from_secs(86400 * deals[0].days_limit as u64))
                .format("%d.%m.%Y")
        );
        // self.send(subject, payload).await?;
        let today = chrono::Local::now().format("%d.%m.%Y").to_string();
        let tpl = NewObjects {
            date: &today,
            payload: &payload,
        };
        self.send(subject, tpl.render().unwrap()).await?;
        Ok(())
    }

    pub async fn send(&self, subject: &str, payload: String) -> Result<()> {
        let username = config().LOGIN.clone();
        let secret = config().PASSWORD.clone();

        let creds = mail_send::Credentials::new(&username, &secret);

        let mut mailer = SmtpClientBuilder::new(&config().SMTP_SERVER, 587)
            .implicit_tls(false)
            .credentials(creds)
            .connect()
            .await?;

        let message = MessageBuilder::new()
            .from(("ДКП бот", config().FROM.as_str()))
            .to(self.receivers.clone())
            .subject(subject)
            .html_body(payload);

        mailer.send(message).await?;
        info!("Email sent");
        Ok(())
    }
    fn prepare_email_content(&self) -> String {
        format!(
            r#"
        <!DOCTYPE html>
<html lang="ru">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title></title>
    </head>
    <body style="font-family: sans-serif">

        <p>Payload: <span>{}</span></p>
    </body>
</html>
        "#,
            "test",
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use log::error;

    #[tokio::test]
    async fn mail_send_test() {
        let email = Email::new();
        let payload = email.prepare_email_content();
        let subject = "Тестовое сообщение от бота";
        let send_result = email.send(&subject, payload).await;

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
