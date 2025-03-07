# HIVIN Bot
HIVIN Bot is a Telegram group management bot that streamlines group administration with features such as automated welcome messages and scheduled broadcasts.

HIVIN Bot (海韵Bot) 是一个 Telegram 群发机器人，提供自动欢迎语、定时推送等功能，让群组管理更轻松高效。

## Key Features 

- 🤖 Easy bot management through chat interface - simple and intuitive
- 👋 Automatic customized welcome messages for new members
- ⏰ Schedule messages to be sent to designated groups
- 💾 Powered by built-in SQLite database - no extra setup required

## 主要功能
- 🤖 通过聊天方式维护机器人，操作简单直观
- 👋 新用户入群自动发送定制欢迎语
- ⏰ 支持定时推送消息到指定群组
- 💾 采用内置 SQLite 数据库，无需额外部署

## Requirements 环境要求
- Rust
- Git

## Quick Start

### 1. Clone | 克隆项目
```bash
git clone https://github.com/bigbug-gg/hivin_bot.git
cd hivin_bot
```
### 2. ENV | 配置环境
```bash
cp .env.bak .env
```

- edit the .env file, set the Telegram Bot Token on TELOXIDE_TOKEN
- 编辑 .env 文件，设置 TELOXIDE_TOKEN 字段为你的 Telegram Bot Token

### 3. RUN | 运行项目

```bash
cargo run
```
**We recommend testing all features after local setup to understand the full range of functionality.**

**建议您本地构建之后尝试下每个功能，以便知道能做那些操作。**

### 4. Demo|使用演示

```bash
/help
```

Return

```
Commands:
/help — Help
/start — Start
/cancel — Cancel
/whoami — Who am i?
```

### History|版本历史
v1.0.0 (2025-03-07)
- 首次发布
- 支持自动欢迎语
- 支持定时推送
- 内置 SQLite 数据库
- 贡献
- 欢迎提交 Issue 和 Pull Request！

### License|许可证
[MIT License]

### How to contact me|联系方式

- Telegram：[@bigbug_gg]
- Email：[bigbug.site@gmail.com]

---
- ⭐️ If you find this project helpful, please star it!
- ⭐️ 如果这个项目对你有帮助，欢迎给个 star！
