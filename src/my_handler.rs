mod admin;
mod callback_query;
mod group_event;
mod poll_message;
mod welcome_message;
mod group_set;

use crate::my_handler::admin::{add_admin_submit, rename_admin_submit};
use crate::my_handler::group_event::handle_new_members;

use crate::{commands, HandlerResult, State};
use log::info;

use teloxide::{
    dptree::case,
    dispatching::dialogue::ErasedStorage,
    dispatching::{UpdateFilterExt, UpdateHandler},
    dptree,
    prelude::*,
};
use crate::my_handler::group_set::handle_group_push_datetime;
use crate::my_handler::poll_message::{add_poll_message, add_poll_message_title};
use crate::my_handler::welcome_message::handle_set_welcome_msg;

/// Create handler
pub fn create() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    dptree::entry()
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, ErasedStorage<State>, State>()
                .endpoint(callback_query::enter),
        )
        .branch(
            Update::filter_message()
                .branch(
                    dptree::filter(|msg: Message| {msg.new_chat_members().is_some()})
                        .endpoint(handle_new_members)
                )
                .enter_dialogue::<Message, ErasedStorage<State>, State>()
                .branch(command_handler())
                .branch(admin_command_handler())
                .branch(dialogue_handler())
                .endpoint(last_branch_handler)
        )
}

fn command_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    dptree::filter(|msg: Message| {
        msg.text()
            .map(|text| text.starts_with('/'))
            .unwrap_or(false)
    })
    .filter_command::<commands::Command>()
    .endpoint(commands::answer::enter)
}

fn admin_command_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    dptree::filter(|msg: Message| {
        msg.text()
            .map(|text| text.starts_with('/'))
            .unwrap_or(false)
    })
    .filter_command::<commands::AdminCommand>()
    .endpoint(commands::start_command::enter)
}

/// Dialogue handler
 fn dialogue_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    info!("Dialogue handler created.");
    Update::filter_message()
        .filter_async(|msg: Message| async move { msg.text().is_some() }) //
        .branch(
            dptree::entry()
                // Message
                .branch(case![State::SetWelcomeMsg].endpoint(handle_set_welcome_msg))

                // Add poll message
                .branch(case![State::AddPollingMsg].endpoint(add_poll_message))
                .branch( case![State::AddPollingTitle(title)].endpoint(add_poll_message_title))

                // Update admin user name
                .branch(case![State::AdminRename(user_id)].endpoint(rename_admin_submit))
                .branch(case![State::AdminAdd].endpoint(add_admin_submit))
                // Group
                .branch(case![State::GroupPushMsg{group_db_id, group_name, msg_db_id}].endpoint(handle_group_push_datetime))
                // other
                .branch(case![State::Menu].endpoint(handle_invalid_command)),
        )
}


async fn handle_invalid_command(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Invalid command. Type /help")
        .await?;
    Ok(())
}
async fn last_branch_handler(bot: Bot, msg: Message) -> HandlerResult {
    if let Some(text) = msg.text() {
        info!("not match text: {}", text);
        if text.starts_with('/') {
            
            bot.send_message(
                msg.chat.id,
                "Unknown command. Use /help to see available commands."
            ).await?;
        } else {
            // 普通消息
            // bot.send_message(msg.chat.id, "I only respond to commands.").await?;
        }
    } else {
        info!("received other type message");
    }

    Ok(())
    
}