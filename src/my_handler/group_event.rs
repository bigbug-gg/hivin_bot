use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::service::{ msg, Db};

pub async fn handle_new_members(bot: Bot, message: Message, db: Db) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(new_members) = message.new_chat_members() {
        for member in new_members {
            if member.is_bot {
                continue;
            }

            let welcome_msg = format!(
                "Welcome {} to the group!",
                member.first_name
            );

            bot.send_message(message.chat.id, welcome_msg).await?;
            let welcome_msg = msg::new(db.clone()).welcome_msg().await;
            bot.send_message(message.chat.id, welcome_msg)
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }
    Ok(())
}