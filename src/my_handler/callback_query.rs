use crate::commands::start_command::admin_menu;
use crate::my_handler::admin::{admin_chose_menu, all_admin, delete_admin, rename_admin};
use crate::my_handler::group_set::{
    group_add_push, group_delete_push, group_msg_choose, group_view_push, show_group_buttons,
    show_group_menu,
};
use crate::my_handler::poll_message::{init_add_poll_message, list_poll_message};
use crate::my_handler::welcome_message::{current_welcome_message, setting_welcome_message};
use crate::service::Db;
use crate::{HandlerResult, MainDialogue, State};
use log::info;
use std::str::FromStr;
use teloxide::payloads::{AnswerCallbackQuerySetters, EditMessageTextSetters};
use teloxide::prelude::Requester;
use teloxide::types::CallbackQuery;
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
        ["cancel", "group"] => {
            show_group_buttons(bot, q.clone(), dialogue, db).await?;
        }
        ["group", "add", "push"] => {
            group_add_push(bot, q.clone(), dialogue, db).await?;
        }
        ["group", "msg", msg_db_id] => {
            group_msg_choose(bot, q.clone(), dialogue, i64::from_str(msg_db_id).unwrap()).await?;
        }
        ["group", "view", "push"] => {
            group_view_push(bot, q.clone(), dialogue, db).await?;
        }
        ["group", "delete", "push", push_id] => {
            group_delete_push(bot, q.clone(), dialogue, db, push_id.parse().unwrap()).await?;
        }
        ["group", group_id, res @ ..] => {
            let group_name = res.join("_");
            show_group_menu(bot, q.clone(), dialogue, group_id, &group_name).await?;
        }

        // Admin list
        ["managers"] => {
            all_admin(bot, q, db).await?;
        }

        // add new
        ["newly", "added"] => {
            let message = q.message.as_ref().unwrap();
            dialogue.update(State::AdminAdd).await?;
            bot.edit_message_text(message.chat().id, message.id(), "Add admin: [ID] [name]:\n")
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
            delete_admin(bot, q, dialogue, db).await?;
        }
        ["admin", "rename"] => {
            rename_admin(bot, q, dialogue).await?;
        }
        ["setting", "welcome", "message"] => {
            setting_welcome_message(bot, q, dialogue).await?;
        }
        ["current", "welcome", "message"] => {
            current_welcome_message(bot, q, db).await?;
        }

        ["add", "poll", "message"] => {
            init_add_poll_message(bot, q, dialogue).await?;
        }

        ["list", "poll", "message"] => {
            list_poll_message(bot, q, db).await?;
        }

        ["cancel"] => {
            let mess = q.message.as_ref().unwrap();
            dialogue.update(State::Menu).await?;
            bot.edit_message_text(mess.chat().id, mess.id(), "end")
                .await?;
            bot.delete_message(mess.chat().id, mess.id()).await?;
        }

        _ => {
            bot.answer_callback_query(q.id)
                .text(format!("Missing actuator callback query: {:?}", parts))
                .await?;
        }
    }
    Ok(())
}
