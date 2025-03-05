use crate::commands::start_command::admin_menu;
use crate::my_handler::admin_callback::{admin_chose_menu, all_admin, delete_admin, rename_admin};
use crate::service::{msg, polling_msg, Db};
use crate::{HandlerResult, MainDialogue, State};
use chrono::NaiveTime;
use log::info;
use teloxide::payloads::{AnswerCallbackQuerySetters, EditMessageTextSetters};
use teloxide::prelude::{Message, Requester};
use teloxide::types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::Bot;

/// Query enter
pub async fn enter(bot: Bot, q: CallbackQuery, dialogue: MainDialogue, db: Db) -> HandlerResult {
    info!("Into callback query handle");
    if q.data.is_none() {
        bot.answer_callback_query(q.id)
            .text("Callback but does not carry any data!")
            .await?;
        return Ok(());
    }

    if q.message.is_none() {
        bot.answer_callback_query(&q.id).text("").await?;
        return Ok(());
    }

    let callback_str = q.data.as_ref().unwrap();
    let parts: Vec<&str> = callback_str.split("_").collect();
    match parts.as_slice() {
        // Choose group
        ["group", group_id] => {
            show_group_buttons(&bot, &q, &dialogue, group_id).await?;
        }

        ["group", "msg", group_id] => {
            show_msg_list(&bot, &q, &dialogue, db, group_id).await?;
        }

        // Admin list
        ["managers"] => {
            all_admin(&bot, &q, &db).await?;
        }

        // add new
        ["newly", "added"] => {
            bot.answer_callback_query(q.id)
                .text("Development...")
                .await?;
        }
        ["back", "admin"] => {
            let mess = q.message.as_ref().unwrap();
            bot.edit_message_text(mess.chat().id, mess.id(), "Choose action")
                .reply_markup(admin_menu())
                .await?;
        }
        ["chosen", "admin", param] => {
            dialogue
                .update(State::AdminChoose(param.to_string()))
                .await?;
            admin_chose_menu(&bot, &q).await?;
        }
        ["admin", "delete"] => {
            delete_admin(&bot, &q, dialogue, &db).await?;
        }
        ["admin", "rename"] => {
            rename_admin(&bot, &q, dialogue).await?;
        }
       
        _ => {
            bot.answer_callback_query(q.id)
                .text(format!("Missing actuator callback query: {:?}", parts))
                .await?;
        }
    }

    Ok(())
}

async fn show_group_buttons(
    bot: &Bot,
    q: &CallbackQuery,
    dialogue: &MainDialogue,
    group_id: &str,
) -> HandlerResult {
    let message = q.message.as_ref().unwrap();
    let message_id = message.id();

    // Create operation buttons
    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback(
            "📲 Add Push",
            format!("addpush_{}_{}", group_id, message_id),
        ),
        InlineKeyboardButton::callback(
            "👀 View Push",
            format!("viewpush_{}_{}", group_id, message_id),
        ),
        InlineKeyboardButton::callback("Cancel", format!("cancel_{}_{}", group_id, message_id)),
    ]]);

    dialogue.update(State::Group).await?;
    bot.edit_message_text(
        message.chat().id,
        message_id,
        format!("Selected group: {}\nPlease choose an operation:", group_id),
    )
    .reply_markup(keyboard)
    .await?;
    Ok(())
}

pub async fn show_msg_list(
    bot: &Bot,
    q: &CallbackQuery,
    dialogue: &MainDialogue,
    db: Db,
    group_id: &str,
) -> HandlerResult {
    let list_msg = msg::new(db.clone()).all().await;

    let mut keyboard_buttons: Vec<Vec<InlineKeyboardButton>> = vec![
        // the back button always front of others.
        vec![InlineKeyboardButton::callback(
            "⬅️ back",
            format!("back_to_ops_{}", group_id),
        )],
    ];

    for msg_info in list_msg {
        keyboard_buttons.push(vec![InlineKeyboardButton::callback(
            msg_info.msg_title,
            format!("pushmsg_{}_{}", group_id, msg_info.id,),
        )]);
    }
    let keyboard = InlineKeyboardMarkup::new(keyboard_buttons);
    let message = q.message.as_ref().unwrap();

    dialogue
        .update(State::GroupChoose {
            group_db_id: group_id.parse().unwrap(),
        })
        .await?;

    bot.edit_message_text(message.chat().id, message.id(), "Select group:\n")
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

pub async fn choose_msg(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MainDialogue,
    msg_id: i64,
) -> HandlerResult {
    let group_id;
    match dialogue.get().await?.unwrap() {
        State::GroupChoose { group_db_id } => group_id = group_db_id,
        _ => return Ok(()),
    }

    dialogue
        .update(State::GroupPushMsg {
            group_db_id: group_id,
            msg_db_id: msg_id,
        })
        .await?;

    let message = q.message.as_ref().unwrap();
    bot.send_message(message.chat().id, "Enter time (HH:MM, e.g. 08:20)\n")
        .await?;
    Ok(())
}

pub async fn set_group_push_time(
    bot: Bot,
    msg: Message,
    dialogue: MainDialogue,
    db: Db,
) -> HandlerResult {
    let time_str = msg.text();
    if time_str.is_none() {
        bot.send_message(msg.chat.id, "Enter time (HH:MM, e.g. 08:20)\n")
            .await?;
        return Ok(());
    }
    let time_str = time_str.unwrap();
    let time_ok = match NaiveTime::parse_from_str(&time_str, "%H:%M") {
        Ok(_) => true,
        Err(_) => false,
    };

    if !time_ok {
        bot.send_message(
            msg.chat.id,
            "Error: Invalid time format. Please use HH:MM (e.g. 08:20)\n",
        )
        .await?;
        return Ok(());
    }

    let group_id;
    let msg_id;
    match dialogue.get().await?.unwrap() {
        State::GroupPushMsg {
            group_db_id,
            msg_db_id,
        } => {
            group_id = group_db_id;
            msg_id = msg_db_id;
        }
        _ => {
            bot.send_message(
                msg.chat.id,
                "Set group message push time: state data error!",
            )
            .await?;
            return Ok(());
        }
    }

    let polling_ser = polling_msg::new(db);

    let insert_id = polling_ser
        .add_polling_msg(msg_id, &group_id.to_string(), time_str)
        .await?;

    bot.send_message(
        msg.chat.id,
        if insert_id > 0 {
            "Successfully inserted."
        } else {
            "Failure inserting."
        },
    )
    .await?;

    dialogue.update(State::Menu).await?;
    Ok(())
}
