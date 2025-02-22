use teloxide::{
    dispatching::{UpdateHandler, UpdateFilterExt},
    dptree,
    prelude::*,
};
use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::dptree::case;
use crate::{State, commands};

pub fn create_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    let command_handler = dptree::filter(|msg: Message| msg.text().map(|text| text.starts_with('/')).unwrap_or(false))
        .filter_command::<commands::Command>()
        .endpoint(commands::answer);

    let dialogue_handler = dptree::entry()
        // 管理员管理相关状态处理
        // .branch(case![State::AddAdmin].endpoint(commands::handle_add_admin))
        // .branch(case![State::ConfirmAddAdmin].endpoint(commands::handle_confirm_add_admin))
        // .branch(case![State::RemoveAdmin].endpoint(commands::handle_remove_admin))
        // .branch(case![State::ConfirmRemoveAdmin].endpoint(commands::handle_confirm_remove_admin))
        // 
        // // 消息管理相关状态处理
        // .branch(case![State::AddPollingMsg].endpoint(commands::handle_add_msg))
        // .branch(case![State::InputTime].endpoint(commands::handle_input_time))
        // .branch(case![State::ConfirmMsg].endpoint(commands::handle_confirm_msg))
        // 
        // // 查看设置状态处理
        // .branch(case![State::ViewSettings].endpoint(commands::handle_view_settings))

        // 菜单状态处理
        .branch(case![State::Menu].endpoint(commands::handle_invalid_command));

    dptree::entry()
        .branch(
            Update::filter_message()
                .branch(
                    dptree::filter(|msg: Message| msg.new_chat_members().is_some())
                        .endpoint(commands::handle_new_members)
                )
                .enter_dialogue::<Message, ErasedStorage<State>, State>()
                .branch(command_handler)
                .branch(dialogue_handler)
        )
}
