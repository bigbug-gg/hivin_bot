use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::payloads::{EditMessageTextSetters, SendMessageSetters};
use crate::{HandlerResult, MainDialogue, State};
use crate::commands::start_command::poll_msg_menu;
use crate::service::{msg, Db};
use crate::service::msg::MsgType;

pub async fn init_add_poll_message(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MainDialogue,
) -> HandlerResult {
    dialogue.update(State::AddPollingMsg).await?;
    let message = q.message.as_ref().unwrap();
    bot.edit_message_text(message.chat().id,  message.id(), "Step 1: Add welcome message:").await?;
    Ok(())
}

/// Step 1: Add the poll message content
pub async fn add_poll_message(
    bot: Bot,
    message: Message,
    dialogue: MainDialogue,
) -> HandlerResult {
    let poll_message_content = message.text().unwrap().trim();
    if poll_message_content.is_empty() {
        bot.send_message(message.chat.id, "").await?;
        return Ok(())
    }

    if let Some(add_msg) = message.text() {
        dialogue
            .update(State::AddPollingTitle(add_msg.to_string()))
            .await?;
        bot.send_message(message.chat.id, "Step 2: Set the message title:")
            .await?;
    } else {
        bot.send_message(message.chat.id, "Input Error").await?;
    }
    Ok(())
}

/// Step 2: Add the poll message title
pub async fn add_poll_message_title(
    bot: Bot,
    message: Message,
    dialogue: MainDialogue,
    db: Db,
) -> HandlerResult {
    let state = dialogue.get().await?.unwrap();
    let message_title = message.text().unwrap().trim();
    if message_title.is_empty() {
        bot.send_message(message.chat.id, "Input Error").await?;
        return Ok(());
    }

    let message_content= match state {
        State::AddPollingTitle(add_msg) => {add_msg},
        _ => {
            bot.send_message(message.chat.id, "Status error, auto reset to default")
                .await?;
            dialogue.update(State::Menu).await?;
            bot.send_message(message.chat.id, "Reset success").await?;
            return Ok(())
        }
    };

    let insert_id = msg::new(db).add_msg(MsgType::Polling, &message_content, message_title).await;
    if insert_id <= 0  {
        bot.send_message(
            message.chat.id,
            "The addition was error, please try again later",
        )
            .await?;
        return Ok(());
    }

    bot.send_message(message.chat.id, format!("[{}] addition was successful!", message_title))
        .reply_markup(poll_msg_menu()).await?;
    Ok(())
}

pub async fn list_poll_message(
    bot: Bot,
    q: CallbackQuery,
    db: Db
) -> HandlerResult {
    let message = q.message.as_ref().unwrap();

    let msg_list = crate::service::msg::new(db).all().await;

    if msg_list.is_empty() {
        bot.edit_message_text(message.chat().id,  message.id(),"No messages set yet")
            .reply_markup(poll_msg_menu()).await?;
        return Ok(())
    }

    let mut list_str = String::from("***List***\n");
    for msg_item in msg_list {
        list_str.push_str("\n");
        list_str.push_str(&format!(
            "{}:\n{}\n",
            msg_item.msg_title,
            msg_item.msg_text
        ));
        list_str.push_str("-------------------")
    }

    bot.edit_message_text(message.chat().id, message.id(), list_str)
        .reply_markup(poll_msg_menu()).await?;
    Ok(())
}