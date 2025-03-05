use log::info;
use crate::service::{user, Db};
use crate::{HandlerResult, MainDialogue, State};
use teloxide::payloads::EditMessageTextSetters;
use teloxide::prelude::{CallbackQuery, Requester};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};
use teloxide::Bot;
use teloxide::dispatching::dialogue::GetChatId;

pub async fn all_admin(bot: &Bot, q: &CallbackQuery, db: &Db) -> HandlerResult {
    info!("Into the all admin dashboard");
    let mut button: Vec<Vec<InlineKeyboardButton>> = vec![
        vec![InlineKeyboardButton::callback("⬅️ Back", "back_admin")]
    ];
    
    let user_ser = user::new(db.clone());
    let admin_list = user_ser.all_admins().await;
    for admin in admin_list {
        button.push(vec![InlineKeyboardButton::callback(
            admin.user_name.to_string(),
            format!("chosen_admin_{}", admin.user_id),
        )])
    }
    
    let message = q.message.as_ref().unwrap();
    bot.edit_message_text(message.chat().id, message.id(), "Admin list:\n")
        .reply_markup(InlineKeyboardMarkup::new(button))
        .await?;
    Ok(())
}

/// ADMIN CHOSE MENU
/// When chosen an admin, display the admin menu
pub async fn admin_chose_menu(bot: &Bot, q: &CallbackQuery) -> HandlerResult {
    let button = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("Delete", "admin_delete"),
        InlineKeyboardButton::callback("Rename", "admin_rename"),
    ]]);
    let message = q.message.as_ref().unwrap();
    bot.edit_message_text(message.chat().id, message.id(), "Chose action:\n")
        .reply_markup(button)
        .await?;
    Ok(())
}

/// Delete admin
pub async fn delete_admin(
    bot: &Bot,
    q: &CallbackQuery,
    dialogue: MainDialogue,
    db: &Db,
) -> HandlerResult {
    let message = q.message.as_ref().unwrap();
    match dialogue.get().await?.unwrap() {
        State::AdminChoose(user_id) => {
            let is_ok = user::new(db.clone()).delete_admin(&user_id).await;
            if is_ok {
                bot.send_message(message.chat().id, "deleted!")
                    .await?;
            } else {
                bot.send_message(message.chat().id, "delete fail")
                    .await?;
            }

            // all_admin representative return back.
            all_admin(bot, q, db).await?;
        }
        _ => {
            bot.send_message(
                message.chat().id,
                "Invalid operation, process aborted",
            )
            .await?;
        }
    }
    dialogue.update(State::Admin).await?;
    Ok(())
}

/// Admin: the name of admin update
pub async fn rename_admin(
    bot: &Bot,
    q: &CallbackQuery,
    dialogue: MainDialogue,
) -> HandlerResult {
    let message = q.message.as_ref().unwrap();
    match dialogue.get().await?.unwrap() {
        State::AdminChoose(user_id) => {
            dialogue.update(State::AdminRename(user_id)).await?;
            bot.send_message(message.chat().id, "New name:\n").await?;
        }
        _ => {
            bot.send_message(
                message.chat().id,
                "Invalid operation, process aborted",
            )
                .await?;
        }
    }
    Ok(())
}

pub async fn rename_admin_submit(
    bot: &Bot,
    msg: &Message,
    dialogue: MainDialogue,
    db: &Db
) -> HandlerResult {
    match dialogue.get().await?.unwrap() {
        State::AdminChoose(user_id) => {
            let name=  msg.text().unwrap();
            let _ = user::new(db.clone()).set_admin_name(&user_id, name).await;
            bot.send_message(msg.chat_id().unwrap(), "Renamed success").await?;
        }
        _ => {
            bot.send_message(
                msg.chat_id().unwrap(),
                "Invalid operation, process aborted",
            )
                .await?;
        }
    }

    dialogue.update(State::Admin).await?;
    Ok(())
}
