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
use teloxide::types::{ChatId,ParseMode};
use teloxide::{dptree, Bot};

pub mod commands;
pub mod my_handler;

pub mod service;

#[derive(Clone, Default, Serialize, Deserialize)]
pub enum State {
    #[default]
    Menu,

    // Admin module
    Admin,
    AdminChoose(String),  // user_id
    AdminRename(String), // // user_id
    AdminAdd,

    // Message module
    AddPollingMsg,           // add the message for poll push.
    AddPollingTitle(String), // add the title for the message.
    SetWelcomeMsg,           // Set the group message when a new user joins and send this.

    // Group module
    Group,
    GroupChoose{group_db_id: i64},
    GroupPushMsg{group_db_id: i64, msg_db_id: i64},
    GroupPushTime,

    // 这个作废
    GroupPush {
        msg_id: String,
        group_id: String,
    }, // Set datetime for the group message push.
}

type MainDialogue = Dialogue<State, ErasedStorage<State>>;

type MainStorage = std::sync::Arc<ErasedStorage<State>>;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::dotenv()
        .map_err(|e| log::error!("Failed to load .env file: {}", e))
        .ok();

    pretty_env_logger::init();

    info!("Starting Telegram bot...");

    // Panic hook
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
            // Waiting 1 minute
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });

    let main_handle = tokio::spawn(async move {
        let storage: MainStorage = SqliteStorage::open("hivin_db.sqlite", Json)
            .await
            .map_err(|e| format!("Failed to open SQLite storage: {}", e))?
            .erase();
        
        info!("Message handler created...");
        Dispatcher::builder(bot_clone, my_handler::create())
            .dependencies(dptree::deps![storage, db_main])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
        Ok(())
    });

    // Tokio select! controls the thread.
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

/// Polling thread enter
async fn poll_task(bot: &Bot, db: Db) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Executing poll task...");
    // Get Datetime HH:mm
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
