use crate::service::{user, Db};
use crate::HandlerResult;
use teloxide::payloads::EditMessageTextSetters;
use teloxide::prelude::{CallbackQuery, Requester};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::Bot;

pub async fn all_admin(bot: &Bot, q: &CallbackQuery, db: &Db) -> HandlerResult {
    let user_ser = user::new(db.clone());
    let button: Vec<Vec<InlineKeyboardButton>> = user_ser
        .all_admins()
        .await
        .iter()
        .map(|admin_data| {
            vec![InlineKeyboardButton::callback(
                admin_data.user_name.capacity().to_string(),
                admin_data.id.to_string(),
            )]
        })
        .collect();

    let message = q.message.as_ref().unwrap();
    bot.edit_message_text(message.chat().id, message.id(), "Admin list:\n")
        .reply_markup(InlineKeyboardMarkup::new(button))
        .await?;
    Ok(())
}
