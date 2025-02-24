use crate::commands::{handle_group_but_callback_query, handle_my_chat_member};
use crate::{commands, State};
use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::dptree::case;
use teloxide::{
    dispatching::{UpdateFilterExt, UpdateHandler},
    dptree,
    prelude::*,
};

pub fn create_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    let command_handler = dptree::filter(|msg: Message| {
        msg.text()
            .map(|text| text.starts_with('/'))
            .unwrap_or(false)
    })
    .filter_command::<commands::Command>()
    .endpoint(commands::answer);

    let dialogue_handler = dptree::entry()
        // 管理员管理相关状态处理
        .branch(case![State::AddAdmin].endpoint(commands::handle_add_admin))
        .branch(case![State::DeleteAdmin].endpoint(commands::handle_delete_admin))
        // 消息管理相关状态处理
        .branch(case![State::SetWelcomeMsg].endpoint(commands::handle_set_welcome_msg))
        .branch(case![State::AddPollingMsg].endpoint(commands::handle_add_polling_msg))
        .branch(case![State::AddPollingTitle(title)].endpoint(commands::handle_add_polling_title))
        // .branch(case![State::InputTime].endpoint(commands::handle_input_time))
        // .branch(case![State::ConfirmMsg].endpoint(commands::handle_confirm_msg))
        //
        // // 查看设置状态处理
        // .branch(case![State::ViewSettings].endpoint(commands::handle_view_settings))
        // 菜单状态处理
        .branch(case![State::Menu].endpoint(commands::handle_invalid_command));

    dptree::entry()
        .branch(Update::filter_my_chat_member().endpoint(handle_my_chat_member))
        .branch(Update::filter_callback_query().endpoint(handle_group_but_callback_query))
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, ErasedStorage<State>, State>()
                .branch(command_handler)
                .branch(dialogue_handler),
        )
}
