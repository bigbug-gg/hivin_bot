pub mod answer;
pub mod start_command;

#[deprecated]
use crate::{
    service::{
        msg::{self, MsgType},
        polling_msg, Db,
    },
    HandlerResult, MainDialogue, State,
};
use chrono::NaiveTime;
use log::info;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Me},
    utils::command::BotCommands,
};

/// Default command
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Commands:")]
pub enum Command {
    #[command(description = "Help")]
    Help,

    #[command(description = "Start")]
    Start,

    #[command(description = "Cancel")]
    Cancel,

    #[command(description = "Who am i?")]
    Whoami,
}

/// Admin command
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Commands:")]
pub enum AdminCommand {
    #[command(description = "📋 Admin list")]
    Admins,

    #[command(description = "✨ Welcome")]
    HiMsg,

    #[command(description = "📝 Add msg")]
    PollMsg,
    
    #[command(description = "📜 Messages")]
    Msg,

    #[command(description = "👥 Groups")]
    Group,
}

pub async fn handle_add_polling_msg(
    bot: Bot,
    msg: Message,
    dialogue: MainDialogue,
) -> HandlerResult {
    if let Some(add_msg) = msg.text() {
        dialogue
            .update(State::AddPollingTitle(add_msg.to_string()))
            .await?;
        bot.send_message(msg.chat.id, "Step 2,setting title:")
            .await?;
    } else {
        bot.send_message(msg.chat.id, "Input Error").await?;
    }
    Ok(())
}

pub async fn handle_add_polling_title(
    bot: Bot,
    msg: Message,
    dialogue: MainDialogue,
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
                bot.send_message(msg.chat.id, "The addition was successful")
                    .await?;

                dialogue
                    .update(State::AddPollingTitle(add_msg.to_string()))
                    .await?;
            } else {
                bot.send_message(
                    msg.chat.id,
                    "The addition was error, please try again later",
                )
                .await?;
            }
            dialogue.update(State::Menu).await?;
        } else {
            bot.send_message(msg.chat.id, "Input Error").await?;
        }
    } else {
        bot.send_message(msg.chat.id, "Status error, auto reset to default")
            .await?;
        dialogue.update(State::Menu).await?;
        bot.send_message(msg.chat.id, "Reset success").await?;
        return Ok(());
    }

    Ok(())
}

pub async fn handle_set_welcome_msg(
    bot: Bot,
    msg: Message,
    dialogue: MainDialogue,
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
                "Welcome message saved. Triggers on new member join.",
            )
            .await?;
        } else {
            bot.send_message(msg.chat.id, "Setting failed. Please retry.")
                .await?;
        }
        dialogue.update(State::Menu).await?;
    } else {
        bot.send_message(msg.chat.id, "").await?;
    }
    Ok(())
}

pub async fn handle_invalid_command(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Invalid command. Type /help")
        .await?;
    Ok(())
}

/// 按钮监听入口
#[deprecated]
pub async fn handle_group_but_callback_query(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MainDialogue,
    db: Db,
) -> HandlerResult {
    info!("Into callback_query handle");

    if q.data.is_none() {
        bot.answer_callback_query(&q.id)
            .text("Empty request")
            .await?;
        return Ok(());
    }

    let data = q.data.as_ref().unwrap();
    if data.starts_with("group_") {
        let group_id = data.replace("group_", "");
        show_group_buttons(&bot, &q, &group_id).await?;
    } else if data.starts_with("group_msg_") {
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
#[deprecated]
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
#[deprecated]
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
    dialogue: MainDialogue,
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
#[deprecated]
pub async fn handle_group_push_callback(
    bot: Bot,
    msg: Message,
    dialogue: MainDialogue,
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

/// Cancel
async fn handle_cancel_callback(bot: &Bot, q: &CallbackQuery) -> HandlerResult {
    if let Some(message) = &q.message {
        bot.answer_callback_query(&q.id).text("已取消").await?;

        bot.edit_message_text(message.chat().id, message.id(), "操作已取消")
            .await?;
    }
    Ok(())
}
