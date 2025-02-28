
use teloxide::Bot;
use teloxide::payloads::EditMessageTextSetters;
use teloxide::prelude::{CallbackQuery, Requester};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::HandlerResult;
use crate::service::{user, Db};

pub struct Admin {
    user: user::User
}

pub fn new(db: Db) -> Admin {
    Admin {
        user: user::new(db.clone())
    }
}

impl Admin {
    pub async fn all_admin(
        &self,
        bot: &Bot,
        q: &CallbackQuery,
    ) -> HandlerResult {
        let button: Vec<Vec<InlineKeyboardButton>> = self.user.all_admins().await.iter().map(|admin_data|{
            vec![InlineKeyboardButton::callback(
                admin_data.user_name.capacity().to_string(),
                admin_data.id.to_string(),
            )]
        }).collect();
        
        let message = q.message.as_ref().unwrap();
        bot.edit_message_text(
            message.chat().id,
            message.id(),
            "Admin list:\n"
        ).reply_markup(InlineKeyboardMarkup::new(button)).await?;
        Ok(())
    }
}


