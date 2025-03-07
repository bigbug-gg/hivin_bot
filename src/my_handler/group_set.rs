//! # Group
//! All operations related to groups are here

use crate::commands::start_command::group_buttons;
use crate::service::{msg, polling_msg, Db};
use crate::{HandlerResult, MainDialogue, State};
use chrono::NaiveTime;
use std::str::FromStr;
use teloxide::payloads::EditMessageTextSetters;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::Bot;

pub async fn show_group_menu(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MainDialogue,
    group_id: &str,
    group_name: &str,
) -> HandlerResult {
    let message = q.message.as_ref().unwrap();
    let message_id = message.id();

    dialogue
        .update(State::GroupChoose {
            group_db_id: i64::from_str(group_id).unwrap(),
            group_name: group_name.to_string(),
        })
        .await?;

    bot.edit_message_text(
        message.chat().id,
        message_id,
        format!("{}\nPlease choose an operation:", group_name),
    )
    .reply_markup(group_menu())
    .await?;
    Ok(())
}

pub fn group_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("📲 Add Push", "group_add_push"),
        InlineKeyboardButton::callback("👀 View Push", "group_view_push"),
        InlineKeyboardButton::callback("Cancel", "cancel_group"),
    ]])
}

/// Show group buttons
pub async fn show_group_buttons(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MainDialogue,
    db: Db,
) -> HandlerResult {
    let message = q.message.as_ref().unwrap();
    match group_buttons(db).await {
        Some(groups) => {
            dialogue.update(State::Group).await?;
            bot.edit_message_text(message.chat().id, message.id(), "Selected group:")
                .reply_markup(groups)
                .await?;
        }
        None => {
            dialogue.update(State::Menu).await?;
            bot.edit_message_text(
                message.chat().id,
                message.id(),
                "The robot has not joined any groups yet!",
            )
            .await?;
        }
    }
    Ok(())
}

/// Group add push
/// Show the message list button
pub async fn group_add_push(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MainDialogue,
    db: Db,
) -> HandlerResult {
    let message = q.message.as_ref().unwrap();

    let (group_db_id, group_name) = match dialogue.get().await?.unwrap() {
        State::GroupChoose {
            group_db_id,
            group_name,
        } => (group_db_id, group_name),
        _ => {
            bot.edit_message_text(
                message.chat().id,
                message.id(),
                "Message is empty. Add content to continue.",
            )
            .await?;
            dialogue.update(State::Menu).await?;
            return Ok(());
        }
    };

    let message_list = msg::new(db).all().await;
    let mut keyboard_buttons: Vec<Vec<InlineKeyboardButton>> =
        vec![vec![InlineKeyboardButton::callback(
            "⬅️ Back",
            format!("group_{}_{}", group_db_id, group_name),
        )]];
    for msg_info in message_list {
        keyboard_buttons.push(vec![InlineKeyboardButton::callback(
            msg_info.msg_title,
            format!("group_msg_{}", msg_info.id,),
        )]);
    }

    let keyboard = InlineKeyboardMarkup::new(keyboard_buttons);
    bot.edit_message_text(
        message.chat().id,
        message.id(),
        "Please specify the message:\n",
    )
    .reply_markup(keyboard)
    .await?;

    Ok(())
}

/// Group add push: choose the message
pub async fn group_msg_choose(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MainDialogue,
    msg_db_id: i64,
) -> HandlerResult {
    let message = q.message.as_ref().unwrap();
    let (group_db_id, group_name) = match dialogue.get().await?.unwrap() {
        State::GroupChoose {
            group_db_id,
            group_name,
        } => (group_db_id, group_name),
        _ => {
            bot.edit_message_text(message.chat().id, message.id(), "Abnormal status, exited!")
                .await?;
            dialogue.update(State::Menu).await?;
            return Ok(());
        }
    };

    dialogue
        .update(State::GroupPushMsg {
            group_db_id,
            group_name,
            msg_db_id,
        })
        .await?;
    bot.edit_message_text(
        message.chat().id,
        message.id(),
        "Time (HH:MM): e.g. 08:20\n",
    )
    .await?;
    Ok(())
}

/// Group add push: set push datetime
pub async fn handle_group_push_datetime(
    bot: Bot,
    msg: Message,
    dialogue: MainDialogue,
    db: Db,
) -> HandlerResult {
    let time_str = msg.text();
    if time_str.is_none() {
        bot.send_message(msg.chat.id, "Time (HH:MM): e.g. 08:20\n")
            .await?;
        return Ok(());
    }
    let time_str = time_str.unwrap();
    let time_ok = match NaiveTime::parse_from_str(&time_str, "%H:%M") {
        Ok(_) => true,
        Err(_) => false,
    };

    if !time_ok {
        bot.send_message(msg.chat.id, "Wrong format. Use HH:MM (e.g. 08:20)\n")
            .await?;
        return Ok(());
    }

    let state = dialogue.get().await?.unwrap();
    match state {
        State::GroupPushMsg {
            group_db_id,
            group_name,
            msg_db_id,
        } => {
            let polling_ser = polling_msg::new(db);

            let insert_id = polling_ser
                .add_polling_msg(msg_db_id, group_db_id, time_str)
                .await?;
            let return_str = if insert_id > 0 { "Success" } else { "Failed" };

            dialogue
                .update(State::GroupChoose {
                    group_db_id,
                    group_name,
                })
                .await?;
            bot.send_message(msg.chat.id, return_str)
                .reply_markup(group_menu())
                .await?;

            return Ok(());
        }
        _ => {
            bot.send_message(msg.chat.id, "Abnormal status, exited!")
                .await?;
        }
    }

    dialogue.update(State::Menu).await?;
    Ok(())
}

/// Group: view the push list
pub async fn group_view_push(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MainDialogue,
    db: Db,
) -> HandlerResult {
    let message = q.message.as_ref().unwrap();
    let state = dialogue.get().await?.unwrap();
    let (group_db_id, group_name) = match state {
        State::GroupChoose {
            group_db_id,
            group_name,
        } => (group_db_id, group_name),
        _ => {
            bot.edit_message_text(message.chat().id, message.id(), "Abnormal status, exited!")
                .await?;
            dialogue.update(State::Menu).await?;
            return Ok(());
        }
    };

    let all_push = polling_msg::new(db).get_group_msgs(group_db_id).await?;

    let mut keyboard_buttons: Vec<Vec<InlineKeyboardButton>> = vec![
        // back front other button.
        vec![InlineKeyboardButton::callback(
            "⬅️ back",
            format!("group_{}_{}", group_db_id, group_name),
        )],
    ];

    for push_info in all_push {
        keyboard_buttons.push(vec![InlineKeyboardButton::callback(
            format!("{}-{}", push_info.send_time, push_info.msg_title),
            format!("group_delete_push_{}", push_info.id,),
        )]);
    }

    let keyboard = InlineKeyboardMarkup::new(keyboard_buttons);
    bot.edit_message_text(message.chat().id, message.id(), "Click to delete:")
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

/// Group: delete the push message.
pub async fn group_delete_push(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MainDialogue,
    db: Db,
    push_id: i64,
) -> HandlerResult {
    let message = q.message.as_ref().unwrap();
    let state = dialogue.get().await?.unwrap();
    let (_group_db_id, _group_name) = match state {
        State::GroupChoose {
            group_db_id,
            group_name,
        } => (group_db_id, group_name),
        _ => {
            bot.edit_message_text(message.chat().id, message.id(), "Abnormal status, exited!")
                .await?;
            dialogue.update(State::Menu).await?;
            return Ok(());
        }
    };

    let is_ok = polling_msg::new(db.clone())
        .delete_polling_msg_by_id(push_id)
        .await?;

    let return_str = if is_ok { "Success" } else { "Failed" };
    bot.edit_message_text(message.chat().id, message.id(), return_str)
        .reply_markup(group_menu())
        .await?;

    Ok(())
}
