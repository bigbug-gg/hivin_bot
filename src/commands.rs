pub mod answer;
pub mod start_command;

use teloxide::utils::command::BotCommands;

/// Default command
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Commands:")]
pub enum Command {
    #[command(description = "Help")]
    Help,

    #[command(description = "Start")]
    Start,

    #[command(description = "Cancel")]
    Cancel,

    #[command(description = "Who am i?")]
    Whoami,
}

/// Admin command
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Commands:")]
pub enum AdminCommand {
    #[command(description = "📋 Admin list")]
    Admins,

    #[command(description = "✨ Group welcome message")]
    HiMsg,

    #[command(description = "⏲️ Group polling message")]
    PollMsg,
    
    #[command(description = "🏢 My groups")]
    Group,
}
