# HIVIN Bot

HIVIN Bot (海韵Bot) 是一个 Telegram 群发机器人，提供自动欢迎语、定时推送等功能，让群组管理更轻松高效。

## 主要功能

- 🤖 通过聊天方式维护机器人，操作简单直观
- 👋 新用户入群自动发送定制欢迎语
- ⏰ 支持定时推送消息到指定群组
- 💾 采用内置 SQLite 数据库，无需额外部署

## 环境要求

- Rust 运行环境
- Git

## 快速开始

### 1. 克隆项目
```bash
git clone https://github.com/bigbug_gg/hivin_bot.git
cd hivin-bot
```
### 2. 配置环境
```bash
cp .env.bak .env
```
编辑 .env 文件，设置 TELOXIDE_TOKEN 字段为你的 Telegram Bot Token

### 3. 运行项目
```bash
cargo run
```

### 版本历史
v1.0.0 (2025-03-07)
首次发布
支持自动欢迎语
支持定时推送
内置 SQLite 数据库
贡献
欢迎提交 Issue 和 Pull Request！

### 许可证
[MIT License]

### 联系方式

- Telegram：[@bigbug_gg]
- Email：[bigbug.site@gmail.com]

---
⭐️ 如果这个项目对你有帮助，欢迎给个 star！