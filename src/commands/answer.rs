use crate::commands::{AdminCommand, Command};
use crate::service::{user, Db};
use crate::{HandlerResult, MainDialogue, State};
use log::{error, info};
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Message, Requester};
use teloxide::types::ParseMode;
use teloxide::utils::command::BotCommands;
use teloxide::Bot;

/// Command enter
///
/// The start command only the admin can open
/// If the system of self is not set anyone is admin the first is admin default.
pub async fn enter(
    bot: Bot,
    msg: Message,
    cmd: Command,
    dialogue: MainDialogue,
    db: Db,
) -> HandlerResult {
    info!("into answer command...");
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .parse_mode(ParseMode::Html)
                .await?;
        }

        Command::Start => {
            start_command_init(bot, msg, db).await?;
        }

        Command::Cancel => {
            dialogue.update(State::Menu).await?;
            bot.send_message(msg.chat.id, "This session is ended.")
                .await?;
        }

        Command::Whoami => {
            let user = msg.from.clone().unwrap();
            let user_id = user.id;
            let user_name = user.username.unwrap_or("noname".to_string());
            bot.send_message(
                msg.chat.id,
                format!("Hi {}, you're user ID is:\n{}", user_name, user_id),
            )
            .await?;
        }
    }
    Ok(())
}

///
/// The start command
///
/// 1. When system has the admin then to go to the belong the admin's module commands.
/// 2. When a user has admin privileges, it can access the admin's module commands.
///
async fn start_command_init(bot: Bot, msg: Message, db: Db) -> HandlerResult {
    if msg.from.is_none() {
        info!("The message didn't come from a person.n't from person");
        bot.send_message(msg.chat.id, "Fail:no from user").await?;
        return Ok(());
    }

    let user_service = user::new(db);
    let user = msg.from.clone().unwrap();

    if user_service.is_admin(&user.id.to_string()).await {
        bot.set_my_commands(AdminCommand::bot_commands()).await?;
        bot.send_message(
            msg.chat.id,
            format!(
                "Dear {}, welcome back! These commands are a service for you:\n{}",
                msg.from.unwrap().username.unwrap_or("noname".to_string()),
                AdminCommand::descriptions()
            ),
        )
        .await?;
        return Ok(());
    }

    // Has admin
    if user_service.has_admin().await {
        bot.send_message(msg.chat.id, "Access restricted to administrators only")
            .await?;
        return Ok(());
    }

    // The first user will become the administrator
    let user = msg.from.unwrap();
    let is_ok = user_service
        .add_admin(
            &user.id.to_string(),
            &user.username.unwrap_or("the one".to_string()),
        )
        .await;

    if is_ok {
        bot.send_message(
            msg.chat.id,
            "Congratulations on becoming an administrator! /start will open a new menu",
        )
        .await?;
    } else {
        error!("Failed to set first admin");
        bot.send_message(msg.chat.id, "Setting administrator failed")
            .await?;
    }

    Ok(())
}
