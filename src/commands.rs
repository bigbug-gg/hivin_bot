use crate::commands::ChatMemberStatus::Banned;
use crate::service::msg::MsgType;
use crate::service::{msg as my_msg, polling_msg};
use crate::service::{msg, user};
use crate::{
    service::{group, Db},
    HandlerResult, MyDialogue, State,
};
use chrono::NaiveTime;
use log::{error, info};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        ChatMember, ChatMemberStatus, InlineKeyboardButton, InlineKeyboardMarkup, Me, ParseMode,
        User,
    },
    utils::command::BotCommands,
};
use teloxide::types::ChatMemberKind;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "支持以下命令:")]
pub enum Command {
    #[command(description = "显示此帮助信息")]
    Help,
    #[command(description = "开始使用")]
    Start,
    #[command(description = "取消当前操作")]
    Cancel,

    #[command(description = "我是谁")]
    Whoami,

    // 管理员管理命令
    #[command(description = "添加新管理员")]
    AddAdmin,

    #[command(description = "删除现有管理员")]
    DeleteAdmin,

    #[command(description = "查看管理员列表")]
    ListAdmins,

    #[command(description = "设置欢迎语")]
    SetWelcomeMsg,

    #[command(description = "添加添加消息")]
    AddPollingMessage,

    #[command(description = "查看消息列表")]
    ListMessages,

    #[command(description = "已加入的群")]
    Group,
}

const MESSAGES: Messages = Messages {
    WELCOME_ADMIN: "欢迎使用! 您已被设置为管理员。\n使用 /help 查看所有命令",
    WELCOME_BACK: "欢迎回来! 使用 /help 查看所有命令",
    NOT_ADMIN: "抱歉，您不是管理员，无法使用此功能。",
    INVALID_USER: "无法获取用户信息，请确保您的账号设置正确。",
    ADMIN_SET_FAILED: "设置管理员失败，请稍后重试。",
    ADD_ADMIN_PROMPT: "输入添加为管理员的用户ID和用户名，空格隔开：",
    CONFIRM_ADD_ADMIN: "确认要将用户 {} 添加为管理员吗？\n回复 'yes' 确认，或 'no' 取消",
    ADMIN_ADDED: "已成功添加管理员！",
    REMOVE_ADMIN_PROMPT: "请输入要移除管理员权限的用户ID:",
    ADMIN_REMOVED: "已成功移除管理员权限！",
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
    let display_name = user_info
        .username
        .clone()
        .unwrap_or_else(|| user_info.first_name.clone());

    if !user_service.has_admin().await {
        handle_first_admin(
            bot,
            chat_id,
            dialogue,
            user_service,
            &user_info.id.to_string(),
            &display_name,
        )
        .await
    } else {
        handle_existing_admin(
            bot,
            chat_id,
            dialogue,
            user_service,
            &user_info.id.to_string(),
        )
        .await
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
            bot.send_message(
                chat_id,
                "欢迎使用! 您已被设置为管理员。\n使用 /help 查看所有命令",
            )
            .await?;
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

#[deprecated]
pub async fn handle_new_members(bot: Bot, msg: Message, db: Db) -> HandlerResult {
    if let Some(new_members) = msg.new_chat_members() {
        for member in new_members {
            if member.is_bot && member.id == bot.get_me().await?.id {
                // 机器人被添加到群组
                let chat_id = msg.chat.id.to_string();
                let chat_title = msg.chat.title().unwrap_or("Unknown Group").to_string();

                info!("Bot was added to group: {} (ID: {})", chat_title, chat_id);

                // 将群组信息保存到数据库
                let group_service = group::new(db.clone());
                match group_service.add_group(&chat_id, &chat_title).await {
                    Ok(_) => {
                        log::info!("Successfully added group to database");

                        // 发送欢迎消息
                        bot.send_message(
                            msg.chat.id,
                            "感谢添加我到群组！\n\n\
                            /help - 查看所有命令",
                        )
                        .await?;
                    }
                    Err(e) => {
                        log::error!("Failed to add group to database: {}", e);
                        bot.send_message(
                            msg.chat.id,
                            "初始化群组设置时发生错误，请稍后重试或联系管理员。",
                        )
                        .await?;
                    }
                }
            }
        }
    }
    Ok(())
}

// 和机器人有关的，都到这里。
pub async fn handle_my_chat_member(
    bot: Bot,
    chat_member: ChatMemberUpdated,
    me: Me,
    db: Db,
) -> HandlerResult {
    // 检查是否与机器人相关
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

            // 只进行数据库清理操作，不尝试发送消息
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

///
/// Command 入口
pub async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Command,
    dialogue: MyDialogue,
    db: Db,
) -> HandlerResult {
    info!("Into answer command...");
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
            bot.send_message(msg.chat.id, "已结束当前对话").await?;
        }
        Command::Whoami => {
            let user = msg.from.clone().unwrap();
            let user_id = user.id;
            let user_name = user.username.unwrap_or("有缘人".to_string());
            bot.send_message(
                msg.chat.id,
                format!("Hi {}, 你的ID是:\n{}", user_name, user_id),
            )
            .await?;
        }
        Command::AddAdmin => {
            let user_service = user::new(db);
            if let Some(from_user) = msg.from {
                if !user_service.is_admin(&from_user.id.to_string()).await {
                    bot.send_message(msg.chat.id, MESSAGES.NOT_ADMIN).await?;
                    return Ok(());
                }

                dialogue.update(State::AddAdmin).await?;
                bot.send_message(msg.chat.id, MESSAGES.ADD_ADMIN_PROMPT)
                    .await?;
            }
        }

        Command::DeleteAdmin => {
            let user_service = user::new(db);
            if let Some(from_user) = msg.from {
                if !user_service.is_admin(&from_user.id.to_string()).await {
                    bot.send_message(msg.chat.id, MESSAGES.NOT_ADMIN).await?;
                    return Ok(());
                }
                dialogue.update(State::DeleteAdmin).await?;
                bot.send_message(msg.chat.id, MESSAGES.REMOVE_ADMIN_PROMPT)
                    .await?;
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
                    let mut message = String::from("***管理员***\n");
                    for admin in admins {
                        message.push_str(&format!(
                            "{}, ID：{}, 创建时间：{} \n",
                            admin.user_name,
                            admin.user_id,
                            admin.created_at.format("%Y-%m-%d %H:%M:%S")
                        ));
                    }
                    bot.send_message(msg.chat.id, message).await?;
                }
            }
        }

        Command::SetWelcomeMsg => {
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
        Command::AddPollingMessage => {
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

        Command::ListMessages => {
            let user_service = user::new(db.clone());
            if let Some(from_user) = msg.from {
                if !user_service.is_admin(&from_user.id.to_string()).await {
                    bot.send_message(msg.chat.id, MESSAGES.NOT_ADMIN).await?;
                    return Ok(());
                }

                let msg_list = my_msg::new(db).all().await;

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

        Command::Group => {
            let g = group::new(db).all().await;
            if g.len() <= 0 {
                bot.send_message(msg.chat.id, "暂未添加群").await?;
                return Ok(());
            }

            // 创建 InlineKeyboardButton 数组
            let mut group_but: Vec<InlineKeyboardButton> = vec![];
            for i in g {
                group_but.push(InlineKeyboardButton::callback(
                    i.group_name,
                    format!("group_{}", i.id),
                ));
            }

            // 将按钮数组包装成矩阵形式
            let keyboard = InlineKeyboardMarkup::new(vec![group_but]);
            bot.send_message(msg.chat.id, "所有群\n")
                .reply_markup(keyboard)
                .await?;
        }

        _ => {
            bot.send_message(msg.chat.id, "Sorry, 该功能待完善...V我50，助力此功能🌚")
                .await?;
        }
    };

    Ok(())
}

pub async fn handle_add_admin(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    db: Db,
) -> HandlerResult {
    if let Some(text) = msg.text() {
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() != 2 {
            bot.send_message(msg.chat.id, MESSAGES.INVALID_FORMAT)
                .await?;
            return Ok(());
        }

        let user_id = parts[0];
        let user_name = parts[1];

        let user_service = user::new(db);
        if user_service.add_admin(user_id, user_name).await {
            bot.send_message(msg.chat.id, MESSAGES.ADMIN_ADDED).await?;
        } else {
            bot.send_message(msg.chat.id, MESSAGES.ADMIN_SET_FAILED)
                .await?;
        }
        dialogue.update(State::Menu).await?;
    }
    Ok(())
}

pub async fn handle_delete_admin(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    db: Db,
) -> HandlerResult {
    if let Some(user_id) = msg.text() {
        let user_service = user::new(db);
        if user_service.cancel_admin(user_id).await {
            bot.send_message(msg.chat.id, MESSAGES.ADMIN_REMOVED)
                .await?;
        } else {
            bot.send_message(msg.chat.id, "移除管理员权限失败，请确认用户ID是否正确。")
                .await?;
        }
        dialogue.update(State::Menu).await?;
    } else {
        bot.send_message(msg.chat.id, MESSAGES.INVALID_FORMAT)
            .await?;
    }
    Ok(())
}

pub async fn handle_add_polling_msg(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {
    if let Some(add_msg) = msg.text() {
        dialogue
            .update(State::AddPollingTitle(add_msg.to_string()))
            .await?;
        bot.send_message(msg.chat.id, "第2步 再设置标题:").await?;
    } else {
        bot.send_message(msg.chat.id, MESSAGES.INVALID_FORMAT)
            .await?;
    }
    Ok(())
}

pub async fn handle_add_polling_title(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    db: Db,
) -> HandlerResult {
    let state = dialogue.get().await?.unwrap();

    if let State::AddPollingTitle(add_msg) = state {
        if let Some(msg_title) = msg.text() {
            let msg_service = msg::new(db);
            if msg_service
                .add_msg(MsgType::Polling, &add_msg, msg_title)
                .await
                > 0
            {
                bot.send_message(
                    msg.chat.id,
                    "新增成功，记得设置消息跟群的关联后，定时推送才生效噢",
                )
                .await?;

                dialogue
                    .update(State::AddPollingTitle(add_msg.to_string()))
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "新增失败，请稍后再试")
                    .await?;
            }
            dialogue.update(State::Menu).await?;
        } else {
            bot.send_message(msg.chat.id, MESSAGES.INVALID_FORMAT)
                .await?;
        }
    } else {
        bot.send_message(msg.chat.id, "状态异常，触发重置状态...")
            .await?;
        dialogue.update(State::Menu).await?;
        bot.send_message(msg.chat.id, "重置状态完成").await?;
        return Ok(());
    }

    Ok(())
}

pub async fn handle_set_welcome_msg(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    db: Db,
) -> HandlerResult {
    if let Some(add_msg) = msg.text() {
        let msg_service = msg::new(db);
        if msg_service
            .add_msg(MsgType::Welcome, add_msg, "welcome_msg")
            .await
            > 0
        {
            bot.send_message(
                msg.chat.id,
                "设置欢迎语成功，欢迎语是在群加入新成员时发送的消息.",
            )
            .await?;
        } else {
            bot.send_message(msg.chat.id, "设置失败，请稍后再试")
                .await?;
        }
        dialogue.update(State::Menu).await?;
    } else {
        bot.send_message(msg.chat.id, MESSAGES.INVALID_FORMAT)
            .await?;
    }
    Ok(())
}

pub async fn handle_invalid_command(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "无效的命令。使用 /help 查看所有可用命令。")
        .await?;
    Ok(())
}

/// 按钮监听入口
pub async fn handle_group_but_callback_query(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    db: Db,
) -> HandlerResult {
    info!("in handle_group_but_callback_query");

    if q.data.is_none() {
        bot.answer_callback_query(&q.id).text("我看了一眼").await?;
        return Ok(());
    }

    let data = q.data.as_ref().unwrap();
    if data.starts_with("group_") {
        let group_id = data.replace("group_", "");
        show_group_buttons(&bot, &q, &group_id).await?;
    } else if data.starts_with("addpush_") {
        handle_add_group_push_callback(&bot, &q, db).await?;
    } else if data.starts_with("pushmsg_") {
        handle_set_push_time_callback(&bot, &q, dialogue).await?;
    } else if data.starts_with("viewpush_") {
        handle_view_group_callback(&bot, &q, db).await?;
    } else if data.starts_with("deletepush_") {
        handle_delete_group_push_callback(&bot, &q, db).await?;
    } else if data.starts_with("cancel_") {
        handle_cancel_callback(&bot, &q).await?;
    } else if data.starts_with("back_to_ops_") {
        // 处理返回操作
        let group_id = data.replace("back_to_ops_", "");
        show_group_buttons(&bot, &q, &group_id).await?;
    }
    Ok(())
}

// 显示操作按钮的函数
async fn show_group_buttons(bot: &Bot, q: &CallbackQuery, group_id: &str) -> HandlerResult {
    bot.answer_callback_query(&q.id).text("已选择群组").await?;

    if let Some(message) = &q.message {
        let message_id = message.id();

        // 创建操作按钮
        let keyboard = InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback(
                "添加推送",
                format!("addpush_{}_{}", group_id, message_id),
            ),
            InlineKeyboardButton::callback(
                "已有推送",
                format!("viewpush_{}_{}", group_id, message_id),
            ),
            InlineKeyboardButton::callback("退出", format!("cancel_{}_{}", group_id, message_id)),
        ]]);

        bot.edit_message_text(
            message.chat().id,
            message_id,
            format!("已选择群组: {}\n请选择操作:", group_id),
        )
        .reply_markup(keyboard)
        .await?;
    }
    Ok(())
}

//
// 群推送,展示消息
// 处理"设置"按钮的回调
// 1. 展示所有的 内容 标题
// 2. 选择其中一个
// 3. 设置时间
// 4. 完成当前操作
async fn handle_add_group_push_callback(bot: &Bot, q: &CallbackQuery, db: Db) -> HandlerResult {
    if q.message.is_none() {
        return Ok(());
    }

    bot.answer_callback_query(&q.id).text("设置消息...").await?;

    let parts: Vec<&str> = q.data.as_ref().unwrap().split('_').collect();
    let group_id = parts[1];

    let message = q.message.as_ref().unwrap();
    let all_msg = msg::new(db).all().await;

    let mut keyboard_buttons: Vec<Vec<InlineKeyboardButton>> = vec![
        // 返回按钮单独一行，放在最前面
        vec![InlineKeyboardButton::callback(
            "⬅️ 返回",
            format!("back_to_ops_{}", group_id),
        )],
    ];

    // 把 消息 设置成按钮
    for msg_info in all_msg {
        keyboard_buttons.push(vec![InlineKeyboardButton::callback(
            msg_info.msg_title,
            format!("pushmsg_{}_{}", group_id, msg_info.id,),
        )]);
    }

    let keyboard = InlineKeyboardMarkup::new(keyboard_buttons);
    bot.edit_message_text(
        message.chat().id,
        message.id(),
        "设置页面\n请选择定时发送消息的群：",
    )
    .reply_markup(keyboard)
    .await?;

    Ok(())
}

///
/// 群推送,选择消息
async fn handle_set_push_time_callback(
    bot: &Bot,
    q: &CallbackQuery,
    dialogue: MyDialogue,
) -> HandlerResult {
    if q.message.is_none() {
        return Ok(());
    }
    bot.answer_callback_query(&q.id).text("设置时间...").await?;

    let parts: Vec<&str> = q.data.as_ref().unwrap().split('_').collect();
    let group_id = parts[1];
    let msg_id = parts[2];

    let message = q.message.as_ref().unwrap();

    dialogue
        .update(State::GroupPush {
            msg_id: msg_id.to_string(),
            group_id: group_id.to_string(),
        })
        .await?;
    bot.send_message(message.chat().id, "请输入推送时间，格式 HH:MM, 如 08:20\n")
        .await?;
    Ok(())
}

///
/// 群推送,设置时间
pub async fn handle_group_push_callback(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    db: Db,
) -> HandlerResult {
    let time_str = msg.text();
    if time_str.is_none() {
        bot.send_message(msg.chat.id, "请输入时间，格式为 HH:MM, 例如: 08:20\n")
            .await?;
        return Ok(());
    }
    let time_str = time_str.unwrap();
    let time_ok = match NaiveTime::parse_from_str(&time_str, "%H:%M") {
        Ok(_) => true,
        Err(_) => false,
    };

    if !time_ok {
        bot.send_message(msg.chat.id, "请输入时间错误，格式为 HH:MM, 例如: 08:20\n")
            .await?;
        return Ok(());
    }

    let state = dialogue.get().await?.unwrap();
    match state {
        State::GroupPush { group_id, msg_id } => {
            let polling_ser = polling_msg::new(db);
            let insert_id = polling_ser
                .add_polling_msg(msg_id.parse().unwrap(), &group_id, time_str)
                .await?;
            bot.send_message(
                msg.chat.id,
                if insert_id > 0 {
                    "设置成功"
                } else {
                    "设置失败"
                },
            )
            .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "状态异常，触发重置状态...")
                .await?;
            bot.send_message(msg.chat.id, "重置状态完成").await?;
        }
    }
    dialogue.update(State::Menu).await?;
    Ok(())
}

/// 群推送，查看已有推送
async fn handle_view_group_callback(bot: &Bot, q: &CallbackQuery, db: Db) -> HandlerResult {
    if q.message.is_none() {
        return Ok(());
    }

    let parts: Vec<&str> = q.data.as_ref().unwrap().split('_').collect();
    let group_id = parts[1];

    let message = q.message.as_ref().unwrap();
    let all_push = polling_msg::new(db).get_group_msgs(group_id).await?;

    let mut keyboard_buttons: Vec<Vec<InlineKeyboardButton>> = vec![
        // 返回按钮单独一行，放在最前面
        vec![InlineKeyboardButton::callback(
            "⬅️ 返回",
            format!("back_to_ops_{}", group_id),
        )],
    ];

    for push_info in all_push {
        keyboard_buttons.push(vec![InlineKeyboardButton::callback(
            format!("{}-{}", push_info.send_time, push_info.msg_title),
            format!("deletepush_{}_{}", group_id, push_info.id,),
        )]);
    }

    let keyboard = InlineKeyboardMarkup::new(keyboard_buttons);
    bot.edit_message_text(
        message.chat().id,
        message.id(),
        "已有推送\n点击可删除对应推送：",
    )
    .reply_markup(keyboard)
    .await?;
    Ok(())
}

/// 群推送，删除已有推送
async fn handle_delete_group_push_callback(bot: &Bot, q: &CallbackQuery, db: Db) -> HandlerResult {
    if q.message.is_none() {
        return Ok(());
    }

    let parts: Vec<&str> = q.data.as_ref().unwrap().split('_').collect();
    let group_id = parts[1];
    let push_id = parts[2];

    let message = q.message.as_ref().unwrap();
    let is_ok = polling_msg::new(db.clone())
        .delete_polling_msg_by_id(push_id.parse().unwrap())
        .await?;

    bot.edit_message_text(
        message.chat().id,
        message.id(),
        if is_ok {
            "删除成功"
        } else {
            "删除失败"
        },
    )
    .await?;

    handle_view_group_callback(&bot, &q, db).await?;
    Ok(())
}

// 处理"取消"按钮的回调
async fn handle_cancel_callback(bot: &Bot, q: &CallbackQuery) -> HandlerResult {
    if let Some(message) = &q.message {
        bot.answer_callback_query(&q.id).text("已取消").await?;

        bot.edit_message_text(message.chat().id, message.id(), "操作已取消")
            .await?;
    }
    Ok(())
}
