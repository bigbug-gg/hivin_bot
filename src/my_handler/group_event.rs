use crate::service::{group, msg, Db};
use crate::HandlerResult;
use log::{error, info};
use teloxide::prelude::*;
use teloxide::types::{ChatMemberStatus, Me, ParseMode};
use teloxide::Bot;

pub async fn handle_new_members(
    bot: Bot,
    message: Message,
    db: Db,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(new_members) = message.new_chat_members() {
        for member in new_members {
            if member.is_bot {
                continue;
            }

            let welcome_msg = format!("Welcome {} to the group!", member.first_name);

            bot.send_message(message.chat.id, welcome_msg).await?;
            let welcome_msg = msg::new(db.clone()).welcome_msg().await;
            bot.send_message(message.chat.id, welcome_msg)
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }
    Ok(())
}

pub async fn handle_my_chat_member(
    bot: Bot,
    chat_member: ChatMemberUpdated,
    me: Me,
    db: Db,
) -> HandlerResult {
    if chat_member.new_chat_member.user.id != me.id {
        return Ok(());
    }

    let chat_id = chat_member.chat.id.to_string();
    let chat_title = chat_member
        .chat
        .title()
        .unwrap_or("Unknown Group")
        .to_string();
    let group_service = group::new(db.clone());

    match chat_member.new_chat_member.status() {
        ChatMemberStatus::Left | ChatMemberStatus::Banned => {
            info!("Bot was removed from chat {}: {}", chat_id, chat_title);
            // delete database info only
            match group_service.delete_group(&chat_id).await {
                Ok(true) => {
                    info!("{} was removed from bot database", chat_title);
                }
                Ok(false) => {
                    info!("{} was not found in bot database", chat_title);
                }
                Err(e) => {
                    error!("Failed to delete group from database: {}", e);
                }
            }
        }

        ChatMemberStatus::Member | ChatMemberStatus::Administrator => {
            info!("Bot was added to chat {}: {}", chat_id, chat_title);
            let message_result = bot
                .send_message(chat_id.clone(), "Hello！\n/help - show all commands...")
                .await;

            match message_result {
                Ok(_) => match group_service.add_group(&chat_id, &chat_title).await {
                    Ok(_) => {
                        info!("Successfully added group to database");
                    }
                    Err(e) => {
                        error!("Failed to add group to database: {}", e);
                        let _ = bot
                            .send_message(chat_id, "Init error!\n/help - show all commands...")
                            .await;
                    }
                },
                Err(e) => {
                    error!("Failed to send welcome message: {}", e);
                }
            }
        }
        _ => {
            info!("Unhandled chat member status update");
        }
    }

    Ok(())
}
