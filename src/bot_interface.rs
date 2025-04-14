use crate::config::config;
use crate::model::deal::{get_house_numbers, get_object_numbers, prepare_response};
use crate::model::sync::sync;
use crate::model::Db;
use log::info;
use std::error::Error;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{dialogue, DpHandlerDescription};
use teloxide::dptree::case;
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use teloxide::types::{KeyboardButton, KeyboardMarkup, KeyboardRemove, ReplyMarkup};

type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;
type MyDialogue = Dialogue<State, InMemStorage<State>>;

pub const PROJECTS: [&str; 2] = ["DNS Сити", "ЖК Формат"];
const OBJECT_TYPES: [&str; 3] = ["Квартиры", "Кладовки", "Машиноместа"];
#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ChooseProject,
    ChooseObjectType {
        project: String,
    },
    ChooseHouseNumber {
        project: String,
        object_type: String,
    },
    ChooseObjectNumber {
        project: String,
        object_type: String,
        house: i32,
    },
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum BotCommand {
    /// Информация по объекту
    Start,
    /// Запрос данных в AmoCRM
    Sync,
    /// Rename object type
    Rename,
}

pub fn bot_handler() -> Handler<'static, DependencyMap, HandlerResult, DpHandlerDescription> {
    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(
            Update::filter_message()
                .filter_command::<BotCommand>()
                .branch(case![BotCommand::Sync].endpoint(sync_handler))
                .branch(case![BotCommand::Start].endpoint(start))
                .branch(case![BotCommand::Rename].endpoint(rename_object_types)),
        )
        .branch(
            Update::filter_message()
                .branch(case![State::ChooseProject].endpoint(receive_project_name))
                .branch(case![State::ChooseObjectType { project }].endpoint(receive_object_type))
                .branch(
                    case![State::ChooseHouseNumber {
                        project,
                        object_type
                    }]
                    .endpoint(receive_house_number),
                )
                .branch(
                    case![State::ChooseObjectNumber {
                        project,
                        object_type,
                        house,
                    }]
                    .endpoint(receive_object_number),
                ),
        )
}

fn make_kbd(step: i32) -> KeyboardMarkup {
    let mut keyboard: Vec<Vec<KeyboardButton>> = vec![];

    let labels: Vec<&str> = if step == 1 {
        PROJECTS.to_vec()
    } else {
        OBJECT_TYPES.to_vec()
    };

    for label in labels.chunks(2) {
        let row = label
            .iter()
            .map(|&item| KeyboardButton::new(item.to_owned()))
            .collect();

        keyboard.push(row);
    }

    KeyboardMarkup::new(keyboard).resize_keyboard()
}

async fn make_house_kbd(project: &str, object_type: &str) -> Option<KeyboardMarkup> {
    let mut keyboard: Vec<Vec<KeyboardButton>> = vec![];

    let labels = get_house_numbers(project, object_type).await;

    info!("LABELS {:?}", labels);

    for label in labels.chunks(8) {
        let row = label
            .iter()
            .map(|&item| KeyboardButton::new(item.to_string()))
            .collect();

        keyboard.push(row);
    }

    match keyboard.is_empty() {
        true => None,
        false => Some(KeyboardMarkup::new(keyboard).resize_keyboard()),
    }
}

async fn sync_handler(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Начат поиск новых сделок...".to_string())
        .await?;
    let results = sync(&bot).await;
    let mut have_no_data = true;
    for res in results {
        match res {
            Ok(data) => {
                if !data.is_empty() {
                    have_no_data = false;
                    for r in data {
                        bot.send_message(msg.chat.id, r.to_string()).await?;
                    }
                }
            }
            Err(e) => {
                let admin_id = ChatId(config().ADMIN_ID);
                bot.send_message(admin_id, e.to_string()).await?;
            }
        }
    }
    if have_no_data {
        bot.send_message(msg.chat.id, "Новых сделок не найдено".to_string())
            .await?;
    }

    Ok(())
}

async fn rename_object_types(bot: Bot, msg: Message) -> HandlerResult {
    let db = Db::new().await;
    let res = db.rename_objects().await;
    match res {
        Ok(_) => {
            bot.send_message(msg.chat.id, "Ok".to_string()).await?;
        }

        Err(e) => {
            let admin_id = ChatId(config().ADMIN_ID);
            bot.send_message(admin_id, e.to_string()).await?;
        }
    }
    Ok(())
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    if let Some(text) = msg.text() {
        if text.starts_with("/start") {
            let keyboard = make_kbd(1);
            bot.send_message(msg.chat.id, "Выберите проект")
                .reply_markup(keyboard)
                .await?;
            dialogue.update(State::ChooseProject).await?;
        }
    }
    Ok(())
}

async fn receive_project_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            if PROJECTS.contains(&text) {
                let keyboard = make_kbd(2);
                bot.send_message(msg.chat.id, "Квартиры или кладовки?")
                    .reply_markup(keyboard)
                    .await?;
                dialogue
                    .update(State::ChooseObjectType {
                        project: text.into(),
                    })
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "Сделайте выбор кнопками")
                    .await?;
            }
        }
        None => {
            bot.send_message(msg.chat.id, "Сделайте выбор кнопками")
                .await?;
        }
    }

    Ok(())
}

async fn receive_object_type(
    bot: Bot,
    dialogue: MyDialogue,
    project: String, // Available from `State::ChooseProject`.
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(object_type) => {
            if OBJECT_TYPES.contains(&object_type) {
                let keyboard_option = make_house_kbd(&project, object_type).await;
                match keyboard_option {
                    Some(keyboard) => {
                        bot.send_message(msg.chat.id, "Выберите номер дома")
                            .reply_markup(keyboard)
                            .await?;
                        dialogue
                            .update(State::ChooseHouseNumber {
                                project,
                                object_type: object_type.into(),
                            })
                            .await?;
                    }
                    None => {
                        bot.send_message(msg.chat.id, "Объектов не обнаружено")
                            .reply_markup(ReplyMarkup::KeyboardRemove(KeyboardRemove::new()))
                            .await?;
                    }
                }
            } else {
                bot.send_message(msg.chat.id, "Сделайте выбор кнопками")
                    .await?;
            }
        }
        _ => {
            bot.send_message(msg.chat.id, "Сделайте выбор кнопками")
                .await?;
        }
    }

    Ok(())
}

async fn receive_house_number(
    bot: Bot,
    dialogue: MyDialogue,
    (project, object_type): (String, String), // Available from `State::ChooseObject`.
    msg: Message,
) -> HandlerResult {
    match msg.text().map(|text| text.parse::<i32>()) {
        Some(Ok(house)) => {
            let houses = get_house_numbers(&project, &object_type).await;
            if houses.contains(&house) {
                let numbers = get_object_numbers(&project, &object_type, house).await;
                if numbers.is_empty() {
                    bot.send_message(msg.chat.id, "Объектов не найдено".to_string())
                        .reply_markup(ReplyMarkup::KeyboardRemove(KeyboardRemove::new()))
                        .await?;
                } else {
                    use std::fmt::Write;
                    let message = numbers.iter().fold(
                        "Найдены объекты с номерами:\n".to_string(),
                        |mut output, b| {
                            let _ = write!(output, "/{}, ", b);
                            output
                        },
                    );
                    bot.send_message(msg.chat.id, message)
                        .reply_markup(ReplyMarkup::KeyboardRemove(KeyboardRemove::new()))
                        .await?;
                    if numbers.len() > 1 {
                        bot.send_message(msg.chat.id, "Выберите номер помещения")
                            .await?;
                        dialogue
                            .update(State::ChooseObjectNumber {
                                project,
                                object_type,
                                house,
                            })
                            .await?;
                    } else {
                        let number = *numbers.first().unwrap();
                        let report = prepare_response(&project, &object_type, house, number).await;
                        bot.send_message(msg.chat.id, report).await?;
                        bot.send_message(msg.chat.id, "Чтобы начать сначала,\n нажмите /start")
                            .reply_markup(ReplyMarkup::KeyboardRemove(KeyboardRemove::new()))
                            .await?;
                        dialogue.exit().await?;
                    }
                };
            } else {
                bot.send_message(msg.chat.id, "Сделайте выбор кнопками")
                    .await?;
            }
        }
        _ => {
            bot.send_message(msg.chat.id, "Сделайте выбор кнопками")
                .await?;
        }
    }

    Ok(())
}

async fn receive_object_number(
    bot: Bot,
    dialogue: MyDialogue,
    (project, object_type, house): (String, String, i32), // Available from `State::ChooseHouseNumber`.
    msg: Message,
) -> HandlerResult {
    if let Some(text) = msg.text() {
        let payload = text.trim_start_matches('/');
        let payload = if payload.contains('@') {
            payload.split('@').collect::<Vec<&str>>()[0]
        } else {
            payload
        };
        match payload.parse::<i32>() {
            Ok(number) => {
                let objects = get_object_numbers(&project, &object_type, house).await;
                if objects.contains(&number) {
                    let report = prepare_response(&project, &object_type, house, number).await;
                    bot.send_message(msg.chat.id, report).await?;
                    if objects.len() == 1 {
                        bot.send_message(msg.chat.id, "Чтобы начать сначала,\n нажмите /start")
                            .reply_markup(ReplyMarkup::KeyboardRemove(KeyboardRemove::new()))
                            .await?;
                        dialogue.exit().await?;
                    }
                }
            }
            _ => {
                bot.send_message(msg.chat.id, "Шаблон: /номер помещения")
                    .await?;
            }
        }
    }

    Ok(())
}
