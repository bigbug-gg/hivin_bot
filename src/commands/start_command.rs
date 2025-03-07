use crate::commands::AdminCommand;
use crate::service::{group, user, Db};
use crate::{HandlerResult, MainDialogue, State};
use log::info;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Message, Requester};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::Bot;

pub async fn enter(
    bot: Bot,
    msg: Message,
    cmd: AdminCommand,
    dialogue: MainDialogue,
    db: Db,
) -> HandlerResult {
    info!("into start command...");

    let user_service = user::new(db.clone());
    let from_user = msg.clone().from.unwrap();

    if !user_service.is_admin(&from_user.id.to_string()).await {
        bot.send_message(msg.chat.id, "Access denied. You are not an administrator.")
            .await?;
        return Ok(());
    }

    match cmd {
        AdminCommand::Admins => {
            bot.send_message(msg.chat.id, "Choose action")
                .reply_markup(admin_menu())
                .await?;
        }
        AdminCommand::HiMsg => {
            bot.send_message(msg.chat.id, "Hi msg")
                .reply_markup(hi_msg_menu())
                .await?;
        }
        AdminCommand::PollMsg => {
            bot.send_message(msg.chat.id, "Poll msg")
                .reply_markup(poll_msg_menu())
                .await?;
        }
        AdminCommand::Group => {
            group_menu(bot, msg.clone(), dialogue, db).await?;
        }
    }
    Ok(())
}

/// Admin menu is the button
///
/// Managers: admin list
/// Newly Added: add news
pub fn admin_menu() -> InlineKeyboardMarkup {
    let admin_button = vec![
        ("👨‍💼 Managers", "managers"),
        ("🆕 Newly Added", "newly_added"),
    ];
    let admin_button: Vec<InlineKeyboardButton> = admin_button
        .into_iter()
        .map(|(button_name, button_call_back)| {
            InlineKeyboardButton::callback(button_name, button_call_back)
        })
        .collect();

    let cancel_button = vec![InlineKeyboardButton::callback("Cancel", "cancel")];
    InlineKeyboardMarkup::new(vec![admin_button, cancel_button])
}

pub fn hi_msg_menu() -> InlineKeyboardMarkup {
    let admin_button = vec![
        ("📄 Current", "current_welcome_message"),
        ("⚙️ Settings", "setting_welcome_message"),
    ];
    let admin_button: Vec<InlineKeyboardButton> = admin_button
        .into_iter()
        .map(|(button_name, button_call_back)| {
            InlineKeyboardButton::callback(button_name, button_call_back)
        })
        .collect();

    let cancel_button = vec![InlineKeyboardButton::callback("Cancel", "cancel")];
    InlineKeyboardMarkup::new(vec![admin_button, cancel_button])
}

pub fn poll_msg_menu() -> InlineKeyboardMarkup {
    // Add List
    let admin_button = vec![
        ("➕ Add", "add_poll_message"),
        ("📝 List", "list_poll_message"),
    ];
    let admin_button: Vec<InlineKeyboardButton> = admin_button
        .into_iter()
        .map(|(button_name, button_call_back)| {
            InlineKeyboardButton::callback(button_name, button_call_back)
        })
        .collect();

    let cancel_button = vec![InlineKeyboardButton::callback("Cancel", "cancel")];
    InlineKeyboardMarkup::new(vec![admin_button, cancel_button])
}

pub async fn group_menu(bot: Bot, msg: Message, dialogue: MainDialogue, db: Db) -> HandlerResult {
    match group_buttons(db).await {
        None => {
            bot.send_message(msg.chat.id, "The robot has not joined any groups yet!")
                .await?;
        }
        Some(keyboard) => {
            dialogue.update(State::Group).await?;
            bot.send_message(msg.chat.id, "All groups\n")
                .reply_markup(keyboard)
                .await?;
        }
    }
    Ok(())
}

pub async fn group_buttons(db: Db) -> Option<InlineKeyboardMarkup> {
    let my_groups = group::new(db).all().await;

    if my_groups.len() <= 0 {
        return None;
    }

    let mut group_but: Vec<Vec<InlineKeyboardButton>> = vec![
        vec![InlineKeyboardButton::callback("Cancel", "cancel")]
    ];
    for i in my_groups {
        group_but.push(vec![InlineKeyboardButton::callback(
            &i.group_name,
            format!("group_{}_{}", i.id, i.group_name),
        )]);
    }
    Some(InlineKeyboardMarkup::new(group_but))
}
