use std::panic;
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
    DeleteAdmin,        // 等待输入要删除的管理员ID
    

    // 消息管理相关状态
    AddPollingMsg,      // 添加定时消息内容
    AddPollingTitle(String),  // 设置标题
    SetWelcomeMsg,      // 设置欢迎语

    // 关联消息和群
    
    // 群
    Group,
    
    InputTime,          // 设置消息发送时间
    ConfirmMsg,         // 确认消息设置

    // 查看当前设置
    ViewSettings,       // 查看当前所有设置
}

type MyDialogue = Dialogue<State, ErasedStorage<State>>;
type MyStorage = std::sync::Arc<ErasedStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 设置更大的栈大小
    let builder = std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024); // 32MB 栈空间

    // 设置 panic hook
    panic::set_hook(Box::new(|panic_info| {
        log::error!("Thread panic: {:?}", panic_info);
    }));

    // 环境变量和日志初始化
    dotenv::dotenv().map_err(|e| log::error!("Failed to load .env file: {}", e)).ok();
    if let Err(e) = pretty_env_logger::try_init() {
        eprintln!("Failed to initialize logger: {}", e);
    }

    log::info!("Starting bot...");

    // 在新线程中运行主要逻辑
    let handle = builder.spawn(|| async {
        let bot = Bot::from_env();

        let storage: MyStorage = SqliteStorage::open("hivin_db.sqlite", Json)
            .await
            .map_err(|e| format!("Failed to open SQLite storage: {}", e))?
            .erase();

        let db = service::new("business.sqlite").await;

        let handler = message_handler::create_handler();

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![storage, db])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    })?;

    // 等待线程完成
    match handle.join() {
        Ok(result) => result.await,
        Err(e) => {
            log::error!("Thread panicked: {:?}", e);
            Err("Thread panic occurred".into())
        }
    }
}

