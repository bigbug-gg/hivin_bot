use log::info;
use crate::commands::{AdminCommand};
use crate::service::{group, user, Db};
use crate::{HandlerResult, MainDialogue, State};
use teloxide::prelude::{Message, Requester};
use teloxide::Bot;
use teloxide::payloads::SendMessageSetters;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::service::msg::MsgType;

pub async fn enter(
    bot: Bot,
    msg: Message,
    cmd: AdminCommand,
    dialogue: MainDialogue,
    db: Db,
) -> HandlerResult {
    info!("into start command...");
    
    let user_service = user::new(db.clone());
    let from_user = msg.from.unwrap();
    if !user_service.is_admin(&from_user.id.to_string()).await {
        bot.send_message(msg.chat.id, "Unauthorized").await?;
        return Ok(());
    }
    
    match cmd {
        AdminCommand::Admins => {
            bot.send_message(
                msg.chat.id,
                "Choose action",
            ).reply_markup(admin_menu()).await?;
        }
        AdminCommand::HiMsg => {
            dialogue.update(State::SetWelcomeMsg).await?;
            bot.send_message(msg.chat.id, "Set welcome message for new members:\n")
                .await?;
        }
        AdminCommand::PollMsg => {
            dialogue.update(State::AddPollingMsg).await?;
            bot.send_message(msg.chat.id, "Step 1: Add welcome text\n").await?;
        }
        AdminCommand::Msg => {
            let msg_list = crate::service::msg::new(db).all().await;

            if msg_list.is_empty() {
                bot.send_message(msg.chat.id, "No messages set yet").await?;
            } else {
                let mut message = String::from("***messages***\n");
                for msg_item in msg_list {
                    message.push_str("\n");
                    message.push_str(&format!(
                        "Type: {}\nTitle:{}\nText:\n{}\n",
                        if msg_item.msg_type == MsgType::Polling {
                            "Push message"
                        } else {
                            "Welcome message"
                        },
                        msg_item.msg_title,
                        msg_item.msg_text
                    ));
                    message.push_str("-------------------\n")
                }
                bot.send_message(msg.chat.id, message).await?;
            }
        }
        AdminCommand::Group => {
            let g = group::new(db).all().await;
            if g.len() <= 0 {
                bot.send_message(msg.chat.id, "No groups joined").await?;
                return Ok(());
            }

            // 创建 InlineKeyboardButton 数组
            let mut group_but: Vec<Vec<InlineKeyboardButton>> = vec![];
            for i in g {
                group_but.push(vec![InlineKeyboardButton::callback(
                    i.group_name,
                    format!("group_{}", i.id),
                )]);
            }

            // 将按钮数组包装成矩阵形式
            let keyboard = InlineKeyboardMarkup::new(group_but);
            bot.send_message(msg.chat.id, "All groups\n")
                .reply_markup(keyboard)
                .await?;
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
        ("🆕 Newly Added", "newly_added")];
    let admin_button: Vec<InlineKeyboardButton> = admin_button.into_iter().map(|(button_name, button_call_back)| {
        InlineKeyboardButton::callback(button_name, button_call_back)
    }).collect();
    InlineKeyboardMarkup::new(vec![admin_button])
}
