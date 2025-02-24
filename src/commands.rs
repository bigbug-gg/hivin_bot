use log::{error, info};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{ParseMode, User},
    utils::command::BotCommands,
};
use teloxide::types::{ChatMemberStatus, InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup, Me};
use crate::service::msg as my_msg;
use crate::service::msg::MsgType;
use crate::service::{msg, user};
use crate::{
    service::{group, Db},
    HandlerResult, MyDialogue, State,
};

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
pub async fn handle_my_chat_member(bot: Bot, chat_member: ChatMemberUpdated, me: Me, db: Db) -> HandlerResult {
    // 检查是否与机器人相关
    if chat_member.new_chat_member.user.id != me.id {
        return Ok(());
    }

    let chat_id = chat_member.chat.id.to_string();
    let chat_title = chat_member.chat.title().unwrap_or("Unknown Group").to_string();
    let group_service = group::new(db.clone());

    match chat_member.new_chat_member.status() {
        ChatMemberStatus::Left | ChatMemberStatus::Banned => {
            info!(
                "Bot was removed from chat {}: {}",
                chat_id, chat_title
            );

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
            info!(
                "Bot was added to chat {}: {}",
                chat_id, chat_title
            );

            // 先尝试发送消息，确认有权限
            let message_result = bot.send_message(
                chat_id.clone(),
                "Hello！\n\n/help - 查看所有命令",
            ).await;

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
                            let _ = bot.send_message(
                                chat_id,
                                "初始化群组设置时发生错误，请稍后重试或联系管理员。",
                            ).await;
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
            bot.send_message(msg.chat.id, "已结束当前对话").await?;
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
        },
        Command::AddPollingMessage => {
            let user_service = user::new(db.clone());
            if let Some(from_user) = msg.from {
                if !user_service.is_admin(&from_user.id.to_string()).await {
                    bot.send_message(msg.chat.id, MESSAGES.NOT_ADMIN).await?;
                    return Ok(());
                }
                dialogue.update(State::AddPollingMsg).await?;
                bot.send_message(msg.chat.id, "第1步 先添加内容:\n")
                    .await?;
            }
        },

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
                            if msg_item.msg_type == MsgType::Polling {"定时推送"} else {"欢迎语"},
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
                // callback_data 用于标识按钮回调
                group_but.push(
                    InlineKeyboardButton::callback(
                        i.group_name,
                        format!("group_{}", i.id) // 添加前缀以便在回调中识别
                    )
                );
            }

            // 将按钮数组包装成矩阵形式
            let keyboard = InlineKeyboardMarkup::new(vec![group_but]);
            bot.send_message(msg.chat.id, "所有群\n")
                .reply_markup(keyboard)
                .await?;
        }

        _ => {
            bot.send_message(msg.chat.id, "Sorry, 该功能待完善...V我50，助力此功能🌚").await?;
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

pub async fn handle_add_polling_msg(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue
) -> HandlerResult {
    if let Some(add_msg) = msg.text() {
        dialogue.update(State::AddPollingTitle(add_msg.to_string())).await?;
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
            if msg_service.add_msg(MsgType::Polling, &add_msg, msg_title).await > 0 {
                bot.send_message(
                    msg.chat.id,
                    "新增成功，记得设置消息跟群的关联后，定时推送才生效噢",
                )
                    .await?;

                dialogue.update(State::AddPollingTitle(add_msg.to_string())).await?;
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
        bot.send_message(msg.chat.id, "状态异常，触发重置状态...").await?;
        dialogue.update(State::Menu).await?;
        bot.send_message(msg.chat.id, "重置状态完成").await?;
        return Ok(())
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
        if msg_service.add_msg(MsgType::Welcome, add_msg, "welcome_msg").await > 0 {
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

pub async fn handle_group_but_callback_query(bot: Bot, q: CallbackQuery) -> HandlerResult {
    info!("in handle_group_but_callback_query");
    if let Some(data) = &q.data {
        if data.starts_with("group_") {
            // 提取群组 ID
            let group_id = data.replace("group_", "");
            show_operation_buttons(&bot, &q, &group_id).await?;
        }
        else if data.starts_with("settings_") {
            handle_settings_callback(&bot, &q).await?;
        }
        else if data.starts_with("settings1_") {
            handle_settings1_callback(&bot, &q).await?;
        }
        else if data.starts_with("settings1_") {
            handle_settings2_callback(&bot, &q).await?;
        }
        else if data.starts_with("view_") {
            handle_view_callback(&bot, &q).await?;
        }
        else if data.starts_with("cancel_") {
            handle_cancel_callback(&bot, &q).await?;
        }
        else if data.starts_with("back_to_ops_") {
            // 处理返回操作
            let group_id = data.replace("back_to_ops_", "");
            show_operation_buttons(&bot, &q, &group_id).await?;
        }
    }
    Ok(())
}

// 显示操作按钮的函数
async fn show_operation_buttons(bot: &Bot, q: &CallbackQuery, group_id: &str) -> HandlerResult {
    bot.answer_callback_query(&q.id)
        .text("已选择群组")
        .await?;

    if let Some(message) = &q.message {
        let message_id = message.id();

        // 创建操作按钮
        let keyboard = InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback(
                "设置",
                format!("settings_{}_{}", group_id, message_id)
            ),
            InlineKeyboardButton::callback(
                "查看",
                format!("view_{}_{}", group_id, message_id)
            ),
            InlineKeyboardButton::callback(
                "取消",
                format!("cancel_{}_{}", group_id, message_id)
            ),
        ]]);

        bot.edit_message_text(
            message.chat().id,
            message_id,
            format!("已选择群组: {}\n请选择操作:", group_id)
        )
            .reply_markup(keyboard)
            .await?;
    }
    Ok(())
}

// 处理"设置"按钮的回调
async fn handle_settings_callback(bot: &Bot, q: &CallbackQuery) -> HandlerResult {
    if let Some(message) = &q.message {
        bot.answer_callback_query(&q.id)
            .text("进入设置...")
            .await?;

        // 从callback data中提取group_id
        let parts: Vec<&str> = q.data.as_ref().unwrap().split('_').collect();
        let group_id = parts[1];

        // 创建设置界面的按钮，包含返回按钮
        let keyboard = InlineKeyboardMarkup::new(vec![
            // 这里可以添加其他设置相关的按钮
            vec![
                InlineKeyboardButton::callback("推送内容设置", format!("setting1_{}", group_id)),
                InlineKeyboardButton::callback("推送时间设置", format!("setting2_{}", group_id)),
            ],
            // 返回按钮单独一行
            vec![InlineKeyboardButton::callback(
                "返回",
                format!("back_to_ops_{}", group_id)
            )],
        ]);

        bot.edit_message_text(
            message.chat().id,
            message.id(),
            "设置页面\n请选择要修改的设置项："
        )
            .reply_markup(keyboard)
            .await?;
    }
    Ok(())
}

async fn handle_settings1_callback(bot: &Bot, q: &CallbackQuery) -> HandlerResult {
    if let Some(message) = &q.message {
        bot.answer_callback_query(&q.id)
            .text("进入推送内容设置...")
            .await?;

        // 从callback data中提取group_id
        let parts: Vec<&str> = q.data.as_ref().unwrap().split('_').collect();
        let group_id = parts[1];

        // 创建设置界面的按钮，包含返回按钮
        let keyboard = InlineKeyboardMarkup::new(vec![
            // 这里可以添加其他设置相关的按钮
            vec![
                InlineKeyboardButton::callback("推送内容设置", format!("setting1_{}", group_id)),
                InlineKeyboardButton::callback("推送时间设置", format!("setting2_{}", group_id)),
            ],
            // 返回按钮单独一行
            vec![InlineKeyboardButton::callback(
                "返回",
                format!("back_to_ops_{}", group_id)
            )],
        ]);

        bot.edit_message_text(
            message.chat().id,
            message.id(),
            "设置页面\n请选择要修改的设置项："
        )
            .reply_markup(keyboard)
            .await?;
    }
    Ok(())
}

async fn handle_settings2_callback(bot: &Bot, q: &CallbackQuery) -> HandlerResult {
    if let Some(message) = &q.message {
        bot.answer_callback_query(&q.id)
            .text("进入设置...")
            .await?;

        // 从callback data中提取group_id
        let parts: Vec<&str> = q.data.as_ref().unwrap().split('_').collect();
        let group_id = parts[1];

        // 创建设置界面的按钮，包含返回按钮
        let keyboard = InlineKeyboardMarkup::new(vec![
            // 这里可以添加其他设置相关的按钮
            vec![
                InlineKeyboardButton::callback("推送内容设置", format!("setting1_{}", group_id)),
                InlineKeyboardButton::callback("推送时间设置", format!("setting2_{}", group_id)),
            ],
            // 返回按钮单独一行
            vec![InlineKeyboardButton::callback(
                "返回",
                format!("back_to_ops_{}", group_id)
            )],
        ]);

        bot.edit_message_text(
            message.chat().id,
            message.id(),
            "设置页面\n请选择要修改的设置项："
        )
            .reply_markup(keyboard)
            .await?;
    }
    Ok(())
}

// 处理"查看"按钮的回调
async fn handle_view_callback(bot: &Bot, q: &CallbackQuery) -> HandlerResult {
    if let Some(message) = &q.message {
        bot.answer_callback_query(&q.id)
            .text("正在查看...")
            .await?;

        let parts: Vec<&str> = q.data.as_ref().unwrap().split('_').collect();
        let group_id = parts[1];

        // 创建查看界面的按钮，包含返回按钮
        let keyboard = InlineKeyboardMarkup::new(vec![
            // 这里可以添加其他查看相关的按钮
            vec![
                InlineKeyboardButton::callback("详细信息", format!("details_{}", group_id)),
                InlineKeyboardButton::callback("统计数据", format!("stats_{}", group_id)),
            ],
            // 返回按钮
            vec![InlineKeyboardButton::callback(
                "返回",
                format!("back_to_ops_{}", group_id)
            )],
        ]);

        bot.edit_message_text(
            message.chat().id,
            message.id(),
            "查看页面\n请选择要查看的内容："
        )
            .reply_markup(keyboard)
            .await?;
    }
    Ok(())
}

// 处理"取消"按钮的回调
async fn handle_cancel_callback(bot: &Bot, q: &CallbackQuery) -> HandlerResult {
    if let Some(message) = &q.message {
        bot.answer_callback_query(&q.id)
            .text("已取消")
            .await?;

        bot.edit_message_text(
            message.chat().id,
            message.id(),
            "操作已取消"
        ).await?;
    }
    Ok(())
}