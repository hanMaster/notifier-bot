use crate::config::config;
use crate::Result;
use log::info;
use mail_send::mail_builder::MessageBuilder;
use mail_send::SmtpClientBuilder;

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

    pub async fn new_objects_notification(&self, deals: Vec<String>) -> Result<()> {
        let subject = "Новые сделки по ДКП";
        let payload = deals.join("<br />");
        self.send(subject, payload).await?;
        Ok(())
    }

    pub async fn send(&self, subject: &str, payload: String) -> Result<()> {
        info!("Sending email with {}", payload);

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

    #[tokio::test]
    async fn send_new() {
        let email = Email::new();
        let deals = vec![r#"
        <div>
        Проект: ЖК Формат<br />
Дом № 14<br />
Тип объекта: Квартиры<br />
№ 52<br />
Тип отделки: Предчистовая<br />
Дата регистрации: 19.02.2025<br />
Передать объект до: 21.03.2025
</div>
        "#.to_string(), r#"
        <div>
        Проект: DNS Сити<br />
Дом № 1<br />
Тип объекта: Кладовки<br />
№ 17<br />
Дата регистрации: 20.03.2025<br />
Передать объект до: 19.04.2025
</div>
        "#.to_string()];
        let res = email.new_objects_notification(deals).await;
        assert!(res.is_ok());
    }
}
