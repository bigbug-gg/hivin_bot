use log::info;
use teloxide::Bot;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::payloads::{EditMessageTextSetters, SendMessageSetters};
use teloxide::prelude::*;
use crate::commands::start_command::hi_msg_menu;
use crate::{HandlerResult, MainDialogue, State};
use crate::service::{msg, Db};

pub async fn handle_set_welcome_msg(
    bot: Bot,
    message: Message,
    dialogue: MainDialogue,
    db: Db,
) -> HandlerResult {
    let welcome_msg = message.text().unwrap().trim();
    
    if welcome_msg.is_empty() {
        bot.send_message(message.chat_id().unwrap(), "Enter message:\n")
            .await?;
        return Ok(());
    }

    let is_ok = msg::new(db).add_welcome_msg(&welcome_msg).await;

    if is_ok {
        bot.send_message(
            message.chat_id().unwrap(),
            "Welcome message saved. Triggers on new member join.",
        )
            .reply_markup(hi_msg_menu())
            .await?;
    } else {
        bot.send_message(message.chat.id, "Setting failed. Please retry.")
            .await?;
    }

    dialogue.update(State::Menu).await?;
    Ok(())
}

pub async fn setting_welcome_message(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MainDialogue,
) -> HandlerResult {
    info!("Into the setting welcome message");
    dialogue.update(State::SetWelcomeMsg).await?;
    let message = q.message.as_ref().unwrap();
    bot.edit_message_text(message.chat().id, message.id(), "Enter welcome message:\n")
        .await?;
    Ok(())
}

pub async fn current_welcome_message(
    bot: Bot,
    q: CallbackQuery,
    db: Db,
) -> HandlerResult {
    info!("Into the current welcome message");
    let welcome_message = msg::new(db).welcome_msg().await;
    let message = q.message.as_ref().unwrap();
    bot.edit_message_text(
        message.chat().id,
        message.id(),
        format!(
            "\
        Current welcome message:\n\
        {welcome_message} \n\
        (💡: bot will send this message to new group members)\n\
    "
        ),
    )
        .reply_markup(hi_msg_menu())
        .await?;
    Ok(())
}