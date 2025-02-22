use serde::{Deserialize, Serialize};
use teloxide::{Bot, dptree};
use teloxide::dispatching::dialogue::{ErasedStorage, SqliteStorage, Storage};
use teloxide::dispatching::dialogue::serializer::Json;
use teloxide::dispatching::Dispatcher;
use teloxide::prelude::Dialogue;

pub mod commands;
pub mod message_handler;
pub mod service;

#[derive(Clone, Default, Serialize, Deserialize)]
pub enum State {
    #[default]
    Menu,
    // 管理员管理相关状态
    AddAdmin,           // 等待输入要添加的管理员ID
    ConfirmAddAdmin,    // 确认添加管理员
    RemoveAdmin,        // 等待输入要删除的管理员ID
    ConfirmRemoveAdmin, // 确认删除管理员
    

    // 消息管理相关状态
    AddPollingMsg,      // 添加定时消息内容
    InputTime,          // 设置消息发送时间
    ConfirmMsg,         // 确认消息设置

    // 查看当前设置
    ViewSettings,       // 查看当前所有设置
}

type MyDialogue = Dialogue<State, ErasedStorage<State>>;
type MyStorage = std::sync::Arc<ErasedStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn run() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env();
    let storage: MyStorage = SqliteStorage::open("hivin_db.sqlite", Json)
        .await
        .unwrap()
        .erase();

    let db = service::new("business.sqlite").await;
    let handler = message_handler::create_handler();

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![storage, db])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
