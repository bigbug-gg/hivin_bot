use log::info;
use crate::commands::{AdminCommand, MESSAGES};
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
    match cmd {
        AdminCommand::Admins => {
            bot.send_message(
                msg.chat.id,
                "Choose action",
            ).reply_markup(admin_menu()).await?;
        }

        AdminCommand::HiMsg => {
            let user_service = user::new(db.clone());
            if let Some(from_user) = msg.from {
                if !user_service.is_admin(&from_user.id.to_string()).await {
                    bot.send_message(msg.chat.id, MESSAGES.NOT_ADMIN).await?;
                    return Ok(());
                }
                dialogue.update(State::SetWelcomeMsg).await?;
                bot.send_message(msg.chat.id, "请输入您要设定的欢迎语：\n")
                    .await?;
            }
        }
        AdminCommand::PollMsg => {
            let user_service = user::new(db.clone());
            if let Some(from_user) = msg.from {
                if !user_service.is_admin(&from_user.id.to_string()).await {
                    bot.send_message(msg.chat.id, MESSAGES.NOT_ADMIN).await?;
                    return Ok(());
                }
                dialogue.update(State::AddPollingMsg).await?;
                bot.send_message(msg.chat.id, "第1步 先添加内容:\n").await?;
            }
        }
        AdminCommand::Msg => {

            let user_service = user::new(db.clone());
            if let Some(from_user) = msg.from {
                if !user_service.is_admin(&from_user.id.to_string()).await {
                    bot.send_message(msg.chat.id, MESSAGES.NOT_ADMIN).await?;
                    return Ok(());
                }

                let msg_list = crate::service::msg::new(db).all().await;

                if msg_list.is_empty() {
                    bot.send_message(msg.chat.id, "还未设置任何消息").await?;
                } else {
                    let mut message = String::from("***消息列表***\n");
                    for msg_item in msg_list {
                        message.push_str("\n");
                        message.push_str(&format!(
                            "类型: {}\n标题：{}\n内容：\n{}\n",
                            if msg_item.msg_type == MsgType::Polling {
                                "定时推送"
                            } else {
                                "欢迎语"
                            },
                            msg_item.msg_title,
                            msg_item.msg_text
                        ));
                        message.push_str("-------------------\n")
                    }
                    bot.send_message(msg.chat.id, message).await?;
                }
            }
        }
        AdminCommand::Group => {
            let g = group::new(db).all().await;
            if g.len() <= 0 {
                bot.send_message(msg.chat.id, "暂未添加群").await?;
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
            bot.send_message(msg.chat.id, "所有群\n")
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
fn admin_menu() -> InlineKeyboardMarkup {
    let admin_button = vec![
        ("👨‍💼 Managers", "managers"),
        ("🆕 Newly Added", "newly_added")];
    let admin_button: Vec<InlineKeyboardButton> = admin_button.into_iter().map(|(button_name, button_call_back)| {
        InlineKeyboardButton::callback(button_name, button_call_back)
    }).collect();
    InlineKeyboardMarkup::new(vec![admin_button])
}