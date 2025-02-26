use crate::service::{polling_msg, Db};
use chrono::Local;
use log::info;
use serde::{Deserialize, Serialize};
use std::panic;
use teloxide::dispatching::dialogue::serializer::Json;
use teloxide::dispatching::dialogue::{ErasedStorage, SqliteStorage, Storage};
use teloxide::dispatching::Dispatcher;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Dialogue, Requester};
use teloxide::types::{ChatId, ParseMode};
use teloxide::{dptree, Bot};

pub mod commands;
pub mod message_handler;
pub mod service;

#[derive(Clone, Default, Serialize, Deserialize)]
pub enum State {
    #[default]
    Menu,
    // 管理员管理相关状态
    AddAdmin,    // 等待输入要添加的管理员ID
    DeleteAdmin, // 等待输入要删除的管理员ID

    // 消息管理相关状态
    AddPollingMsg,           // 添加定时消息内容
    AddPollingTitle(String), // 设置标题
    SetWelcomeMsg,           // 设置欢迎语

    // 群
    GroupPush {
        msg_id: String,
        group_id: String,
    }, // 设置消息发送时间
}

type MyDialogue = Dialogue<State, ErasedStorage<State>>;
type MyStorage = std::sync::Arc<ErasedStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 环境变量和日志初始化
    dotenv::dotenv()
        .map_err(|e| log::error!("Failed to load .env file: {}", e))
        .ok();

    pretty_env_logger::init();
    info!("Starting Telegram bot...");

    // 设置 panic hook
    panic::set_hook(Box::new(|panic_info| {
        log::error!("Thread panic: {:?}", panic_info);
    }));

    let bot = Bot::from_env();
    let bot_clone = bot.clone();
    let bot_poll = bot.clone();

    let db = service::new("business.sqlite").await;
    let db_main = db.clone();
    let db_poll = db.clone();

    let poll_handle = tokio::spawn(async move {
        loop {
            match poll_task(&bot_poll, db_poll.clone()).await {
                Ok(_) => log::info!("Poll task completed successfully"),
                Err(e) => log::error!("Poll task error: {:?}", e),
            }
            // 等待1分钟
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });
    // 在新线程中运行主要逻辑
    let main_handle = tokio::spawn(async move {
        let storage: MyStorage = SqliteStorage::open("hivin_db.sqlite", Json)
            .await
            .map_err(|e| format!("Failed to open SQLite storage: {}", e))?
            .erase();

        let handler = message_handler::create_handler();

        info!("Message handler created...");
        Dispatcher::builder(bot_clone, handler)
            .dependencies(dptree::deps![storage, db_main])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
        Ok(())
    });

    // 等待两个任务
    tokio::select! {
        result = poll_handle => {
            log::error!("Poll task ended: {:?}", result);
            Err("Poll task ended unexpectedly".into())
        }
        result = main_handle => {
            result.unwrap_or_else(|e| {
                log::error!("Main thread panicked: {:?}", e);
                Err("Thread panic occurred".into())
            })
        }
    }
}

async fn poll_task(bot: &Bot, db: Db) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Executing poll task...");
    // 获取当前时间，格式化为 HH:mm
    let current_time = Local::now().format("%H:%M").to_string();
    info!("Executing poll task at {}...", current_time);

    let push_data = polling_msg::new(db)
        .get_polling_msgs_by_time(&current_time)
        .await;

    if let Err(_) = push_data {
        return Ok(());
    }

    let push_data = push_data?;
    for push_msg in push_data {
        let group_id: i64 = push_msg.group_id.parse()?;
        info!("Push group_id is: {:?}", push_msg);
        bot.send_message(ChatId(group_id), push_msg.msg_text)
            .parse_mode(ParseMode::MarkdownV2)
            .await?;

        info!(
            "Successfully sent message to group {} at {}",
            group_id, current_time
        );
    }
    Ok(())
}
