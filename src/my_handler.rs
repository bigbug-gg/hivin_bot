mod admin_callback;
pub mod callback_query;
pub mod group;

use crate::{commands, State};
use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::dptree::{case};
use teloxide::{dispatching::{UpdateFilterExt, UpdateHandler}, dptree, prelude::*};
use crate::my_handler::group::{handle_member_update, handle_my_chat_member};

/// Create handler
pub fn create() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    dptree::entry()
        .branch(Update::filter_chat_member().endpoint(handle_member_update))
        .branch(Update::filter_my_chat_member().endpoint(handle_my_chat_member))
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, ErasedStorage<State>, State>()
                .endpoint(callback_query::enter),
        )
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, ErasedStorage<State>, State>()
                .branch(command_handler())
                .branch(admin_command_handler())
                .branch(dialogue_handler())
        )
}

fn command_handler() ->UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>>{
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
    dptree::entry()
        // Message
        .branch(case![State::SetWelcomeMsg].endpoint(commands::handle_set_welcome_msg))
        .branch(case![State::AddPollingMsg].endpoint(commands::handle_add_polling_msg))
        .branch(case![State::AddPollingTitle(title)].endpoint(commands::handle_add_polling_title))
        
        // Update admin user name
        // .branch(case![State::AdminRename(name)].endpoint(rename_admin_submit))

        // Group
        .branch(case![State::Group].endpoint(commands::handle_group_push_callback))

        // other
        .branch(case![State::Menu].endpoint(commands::handle_invalid_command))
}
