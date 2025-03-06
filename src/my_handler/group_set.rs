use chrono::NaiveTime;
use crate::{HandlerResult, MainDialogue, State};
use teloxide::payloads::EditMessageTextSetters;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::Bot;
use crate::service::{msg, polling_msg, Db};

pub async fn show_group_buttons(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MainDialogue,
    group_id: &str,
    group_name: &str,
) -> HandlerResult {
    
    let message = q.message.as_ref().unwrap();
    let message_id = message.id();

    // Create operation buttons
    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback(
            "📲 Add Push",
            "group_add_push",
        ),
        InlineKeyboardButton::callback(
            "👀 View Push",
            "group_view_push",
        ),
        InlineKeyboardButton::callback("Cancel", format!("cancel_{}_{}", group_id, message_id)),
    ]]);

    dialogue.update(State::Group).await?;
    bot.edit_message_text(
        message.chat().id,
        message_id,
        format!("Selected group: {}\nPlease choose an operation:", group_name),
    )
    .reply_markup(keyboard)
    .await?;
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
