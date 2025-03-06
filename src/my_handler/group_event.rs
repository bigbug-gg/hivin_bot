use log::{error, info};
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::{ChatMemberKind, ChatMemberStatus, Me};
use crate::HandlerResult;
use crate::service::{group, msg, Db};

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

            // 先尝试发送消息，确认有权限
            let message_result = bot
                .send_message(chat_id.clone(), "Hello！\n\n/help - 查看所有命令")
                .await;

            match message_result {
                Ok(_) => {
                    // 消息发送成功后再保存群组信息
                    match group_service.add_group(&chat_id, &chat_title).await {
                        Ok(_) => {
                            info!("Successfully added group to database");
                        }
                        Err(e) => {
                            error!("Failed to add group to database: {}", e);
                            // 可以尝试发送错误消息，但要注意处理可能的错误
                            let _ = bot
                                .send_message(
                                    chat_id,
                                    "初始化群组设置时发生错误，请稍后重试或联系管理员。",
                                )
                                .await;
                        }
                    }
                }
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


pub async fn handle_member_update(bot: Bot, member_update: ChatMemberUpdated, db: Db) -> HandlerResult {
    let chat_id = member_update.chat.id;

    let user = member_update.from;
    match member_update.new_chat_member.kind {
        ChatMemberKind::Owner(_) => {
            bot.send_message(
                chat_id,
                format!("{} 是群主", user.full_name()),
            ).await?;
        },

        ChatMemberKind::Administrator(_) => {
            bot.send_message(
                chat_id,
                format!("{} 成为管理员", user.full_name()),
            ).await?;
        },

        ChatMemberKind::Member => {
            let welcome = msg::new(db).welcome_msg().await;
            bot.send_message(
                chat_id,
                welcome
            ).await?;
        },

        ChatMemberKind::Restricted(restricted) => {
            bot.send_message(
                chat_id,
                format!("{} 被限制", user.full_name()),
            ).await?;
        },

        ChatMemberKind::Left => {
            bot.send_message(
                chat_id,
                "成员离开群组"
            ).await?;
        },

        ChatMemberKind::Banned(banned) => {
            bot.send_message(
                chat_id,
                format!("{} 被封禁", user.full_name())
            ).await?;
        }
    }

    Ok(())
}
