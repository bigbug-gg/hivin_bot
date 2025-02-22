use log::{error, info};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{ParseMode, User},
    utils::command::BotCommands,
};
use teloxide::dptree::map;
use crate::{HandlerResult, MyDialogue, State, service::{Db, group}};
use crate::service::user;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "支持以下命令:")]
pub enum Command {
    #[command(description = "显示此帮助信息")]
    Help,
    #[command(description = "开始使用")]
    Start,
    #[command(description = "取消当前操作")]
    Cancel,

    // 管理员管理命令
    #[command(description = "添加新管理员")]
    AddAdmin,
    #[command(description = "删除现有管理员")]
    RemoveAdmin,
    #[command(description = "查看管理员列表")]
    ListAdmins,

    // 消息管理命令
    #[command(description = "设置定时消息")]
    SetMessage,
    #[command(description = "查看当前消息设置")]
    ViewSettings,
}

const MESSAGES: Messages = Messages {
    WELCOME_ADMIN: "欢迎使用! 您已被设置为管理员。\n使用 /help 查看所有命令",
    WELCOME_BACK: "欢迎回来! 使用 /help 查看所有命令",
    NOT_ADMIN: "抱歉，您不是管理员，无法使用此功能。",
    INVALID_USER: "无法获取用户信息，请确保您的账号设置正确。",
    ADMIN_SET_FAILED: "设置管理员失败，请稍后重试。",
    ADD_ADMIN_PROMPT: "请输入要添加为管理员的用户ID和用户名，格式: <user_id> <user_name>",
    CONFIRM_ADD_ADMIN: "确认要将用户 {} 添加为管理员吗？\n回复 'yes' 确认，或 'no' 取消",
    ADMIN_ADDED: "已成功添加管理员！",
    REMOVE_ADMIN_PROMPT: "请输入要移除管理员权限的用户ID:",
    ADMIN_REMOVED: "已成功移除管理员权限！",
    ADMIN_LIST_HEADER: "当前管理员列表:\n",
    ADMIN_LIST_ITEM: "ID: {} | 用户名: {} | 添加时间: {}",
    INVALID_FORMAT: "输入格式错误，请重新输入。",
    NO_ADMINS: "当前没有管理员。",
};

struct Messages {
    WELCOME_ADMIN: &'static str,
    WELCOME_BACK: &'static str,
    NOT_ADMIN: &'static str,
    INVALID_USER: &'static str,
    ADMIN_SET_FAILED: &'static str,
    ADD_ADMIN_PROMPT: &'static str,
    CONFIRM_ADD_ADMIN: &'static str,
    ADMIN_ADDED: &'static str,
    REMOVE_ADMIN_PROMPT: &'static str,
    ADMIN_REMOVED: &'static str,
    ADMIN_LIST_HEADER: &'static str,
    ADMIN_LIST_ITEM: &'static str,
    INVALID_FORMAT: &'static str,
    NO_ADMINS: &'static str,
}

async fn handle_start_command(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    user_service: user::User,
) -> HandlerResult {
    match msg.from.clone() {
        Some(user_info) => {
            handle_user_start(bot, msg.chat.id, dialogue, user_service, user_info).await
        }
        None => {
            bot.send_message(msg.chat.id, MESSAGES.INVALID_USER).await?;
            Ok(())
        }
    }
}

async fn handle_user_start(
    bot: Bot,
    chat_id: ChatId,
    dialogue: MyDialogue,
    user_service: user::User,
    user_info: User,
) -> HandlerResult {
    let display_name = user_info.username
        .clone()
        .unwrap_or_else(|| user_info.first_name.clone());

    if !user_service.has_admin().await {
        handle_first_admin(bot, chat_id, dialogue, user_service, &user_info.id.to_string(), &display_name).await
    } else {
        handle_existing_admin(bot, chat_id, dialogue, user_service, &user_info.id.to_string()).await
    }
}

async fn handle_first_admin(
    bot: Bot,
    chat_id: ChatId,
    dialogue: MyDialogue,
    user_service: user::User,
    user_id: &str,
    display_name: &str,
) -> HandlerResult {
    info!("Setting first admin: User ID {}", user_id);

    match user_service.add_admin(user_id, display_name).await {
        true => {
            info!("Successfully set first admin: {}", user_id);
            dialogue.update(State::Menu).await?;
            bot.send_message(chat_id, MESSAGES.WELCOME_ADMIN).await?;
        }
        false => {
            error!("Failed to set first admin: {}", user_id);
            bot.send_message(chat_id, MESSAGES.ADMIN_SET_FAILED).await?;
        }
    }
    Ok(())
}

async fn handle_existing_admin(
    bot: Bot,
    chat_id: ChatId,
    dialogue: MyDialogue,
    user_service: user::User,
    user_id: &str,
) -> HandlerResult {
    if user_service.is_admin(user_id).await {
        dialogue.update(State::Menu).await?;
        bot.send_message(chat_id, MESSAGES.WELCOME_BACK).await?;
    } else {
        bot.send_message(chat_id, MESSAGES.NOT_ADMIN).await?;
    }

    Ok(())
}

pub async fn handle_new_members(
    bot: Bot,
    msg: Message,
    db: Db,
) -> HandlerResult {
    if let Some(new_members) = msg.new_chat_members() {
        for member in new_members {
            if member.is_bot && member.id == bot.get_me().await?.id {
                // 机器人被添加到群组
                let chat_id = msg.chat.id.to_string();
                let chat_title = msg.chat.title().unwrap_or("Unknown Group").to_string();

                log::info!(
                    "Bot was added to group: {} (ID: {})", 
                    chat_title,
                    chat_id
                );

                // 将群组信息保存到数据库
                let group_service = group::new(db.clone());
                match group_service.add_group(&chat_id, &chat_title).await {
                    Ok(_) => {
                        log::info!("Successfully added group to database");

                        // 发送欢迎消息
                        bot.send_message(
                            msg.chat.id,
                            "感谢添加我到群组！\n\n\
                            默认情况下，投票提醒和欢迎消息功能已开启。\n\
                            使用以下命令管理功能：\n\
                            /mute_polling - 关闭投票提醒\n\
                            /unmute_polling - 开启投票提醒\n\
                            /mute_welcome - 关闭欢迎消息\n\
                            /unmute_welcome - 开启欢迎消息\n\
                            /help - 查看所有命令"
                        ).await?;
                    }
                    Err(e) => {
                        log::error!("Failed to add group to database: {}", e);
                        bot.send_message(
                            msg.chat.id,
                            "初始化群组设置时发生错误，请稍后重试或联系管理员。"
                        ).await?;
                    }
                }
            }
        }
    }
    Ok(())
}

pub async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Command,
    dialogue: MyDialogue,
    db: Db,
) -> HandlerResult {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Command::Start => {
            let user_service = user::new(db);
            handle_start_command(bot, msg, dialogue, user_service).await?;
        }
        Command::Cancel => {
            dialogue.update(State::Menu).await?;
            bot.send_message(msg.chat.id, "已取消当前操作").await?;
        }
        
        Command::AddAdmin => {
            let user_service = user::new(db);
            if let Some(from_user) = msg.from {
                if !user_service.is_admin(&from_user.id.to_string()).await {
                    bot.send_message(msg.chat.id, MESSAGES.NOT_ADMIN).await?;
                    return Ok(());
                }
                dialogue.update(State::AddAdmin).await?;
                bot.send_message(msg.chat.id, MESSAGES.ADD_ADMIN_PROMPT).await?;
            }
        }
        Command::RemoveAdmin => {
            let user_service = user::new(db);
            if let Some(from_user) = msg.from() {
                if !user_service.is_admin(&from_user.id.to_string()).await {
                    bot.send_message(msg.chat.id, MESSAGES.NOT_ADMIN).await?;
                    return Ok(());
                }
                dialogue.update(State::RemoveAdmin).await?;
                bot.send_message(msg.chat.id, MESSAGES.REMOVE_ADMIN_PROMPT).await?;
            }
        }
        
        Command::ListAdmins => {
            let user_service = user::new(db);
            if let Some(from_user) = msg.from {
                if !user_service.is_admin(&from_user.id.to_string()).await {
                    bot.send_message(msg.chat.id, MESSAGES.NOT_ADMIN).await?;
                    return Ok(());
                }

                let admins = user_service.all_admins().await;
                if admins.is_empty() {
                    bot.send_message(msg.chat.id, MESSAGES.NO_ADMINS).await?;
                } else {
                    let mut message = String::from(MESSAGES.ADMIN_LIST_HEADER);
                    for admin in admins {
                        message.push_str(&format!(
                            MESSAGES.ADMIN_LIST_ITEM,
                            admin.user_id,
                            admin.user_name,
                            admin.created_at.format("%Y-%m-%d %H:%M:%S")
                        ));
                        message.push('\n');
                    }
                    bot.send_message(msg.chat.id, message).await?;
                }
            }
        }
        
        _ => {}
    };

    Ok(())
}
pub async fn handle_add_admin(bot: Bot, msg: Message, dialogue: MyDialogue, db: Db) -> HandlerResult {
    if let Some(text) = msg.text() {
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() != 2 {
            bot.send_message(msg.chat.id, MESSAGES.INVALID_FORMAT).await?;
            return Ok(());
        }

        let user_id = parts[0];
        let user_name = parts[1];

        let user_service = user::new(db);
        if user_service.add_admin(user_id, user_name).await {
            bot.send_message(msg.chat.id, MESSAGES.ADMIN_ADDED).await?;
        } else {
            bot.send_message(msg.chat.id, MESSAGES.ADMIN_SET_FAILED).await?;
        }
        dialogue.update(State::Menu).await?;
    }
    Ok(())
}

pub async fn handle_remove_admin(bot: Bot, msg: Message, dialogue: MyDialogue, db: Db) -> HandlerResult {
    if let Some(user_id) = msg.text() {
        let user_service = user::new(db);
        if user_service.cancel_admin(user_id).await {
            bot.send_message(msg.chat.id, MESSAGES.ADMIN_REMOVED).await?;
        } else {
            bot.send_message(msg.chat.id, "移除管理员权限失败，请确认用户ID是否正确。").await?;
        }
        dialogue.update(State::Menu).await?;
    } else {
        bot.send_message(msg.chat.id, MESSAGES.INVALID_FORMAT).await?;
    }
    Ok(())
}

pub async fn handle_invalid_command(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "无效的命令。使用 /help 查看所有可用命令。",
    )
        .await?;
    Ok(())
}