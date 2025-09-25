use chrono::Local;
use std::sync::Arc;
use std::time::Duration;

use teloxide::prelude::*;
use tokio::sync::mpsc;
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};

use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

/// 日志输出格式
#[derive(Clone)]
pub enum TelegramFormat {
    Text,
    Markdown,
    Json,
    Template(&'static str),
}

/// 队列异步发送器
#[derive(Clone)]
struct TelegramSender {
    sender: mpsc::Sender<(String, Option<teloxide::types::ParseMode>)>,
}

impl TelegramSender {
    pub fn new(bot: Arc<Bot>, chat_ids: Vec<i64>) -> Self {
        //   let (ftx,frx ) = futures::channel::mpsc::channel::<(String,Option<teloxide::types::ParseMode>)>(100);
        let (tx, mut rx) = mpsc::channel::<(String, Option<teloxide::types::ParseMode>)>(100);
        let bot_clone = bot.clone();
     
        tokio::spawn(async move {
            while let Some((msg, parse_mode)) = rx.recv().await {
                 let chat_id = chat_ids.clone();
                for chat_id in chat_id {
                       let mut req = bot_clone.send_message(ChatId(chat_id), msg.to_owned());
                if let Some(pm) = parse_mode {
                    req = req.parse_mode(pm);
                }
                if let Err(_) = req.await {
                    // eprintln!("Failed to send log to Telegram: {}", err);
                    tokio::time::sleep(Duration::from_secs(60)).await; // 等待60秒后重试
                }
                }
            }
        });

        Self { sender: tx }
    }

    async fn send(&self, msg: String, parse_mode: Option<teloxide::types::ParseMode>) {
        let _ = self.sender.send((msg, parse_mode)).await;
    }
}

/// Telegram Layer
#[derive(Clone)]
pub struct TelegramLayer {
    sender: TelegramSender,
    format: TelegramFormat,
    tag: Vec<String>,
    unknown: String,
}


impl TelegramLayer {
    pub fn builder() -> TelegramLayerBuilder {
        TelegramLayerBuilder::default()
    }
}

/// 提取 event message
struct MessageVisitor {
    output: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.output.push_str(&format!("{:?}", value));
        }
    }
}

use std::collections::HashMap;
use tracing::Level;

use lazy_static::lazy_static;

use crate::escape_markdown_v2;

lazy_static! {
    static ref LEVEL_EMOJIS: HashMap<Level, &'static str> = {
        let mut map = HashMap::new();
        map.insert(Level::ERROR, "❌");
        map.insert(Level::WARN, "⚠️");
        map.insert(Level::INFO, "ℹ️");
        map.insert(Level::DEBUG, "🔍");
        map.insert(Level::TRACE, "📝");
        map
    };
}

impl<S> Layer<S> for TelegramLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = MessageVisitor {
            output: String::new(),
        };
        event.record(&mut visitor);

        if visitor.output.is_empty() {
            return;
        }
        // 允许tag 标记日志
        if self.tag.len() > 0 {
            let mut flag = false;
            for tag in &self.tag {
                if visitor.output.contains(tag) {
                    flag = true;
                    break;
                }
            }
            if !flag {
                return;
            }
        }

        let meta = event.metadata();
        let line = meta.line();
        let file = meta.file();
        let module = meta.module_path();
        let level = meta.level();
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let emoji = LEVEL_EMOJIS.get(&level).copied().unwrap_or(&self.unknown);

        let (msg, parse_mode) = match self.format {
            TelegramFormat::Text => (format!("{} [{}] {}", emoji, now, visitor.output), None),
            TelegramFormat::Markdown => {
                        let escaped_output = escape_markdown_v2(&visitor.output);
                        let file = file.unwrap_or(&self.unknown).replace("\\", "/");
                        let line = line.unwrap_or(0);
                        let module = module.unwrap_or(&self.unknown);
                        (
                            format!(
                                "```\n{emoji} [{}] {}:{} {} {} [{level}]\n```",
                                now, module,line, file, escaped_output
                            ),
                            Some(teloxide::types::ParseMode::MarkdownV2),
                        )
                    }
            TelegramFormat::Json => {
                        let json = format!(
                            r#"``` {{"time": "{}", "emoji": "{}", "msg": "{}", "level": "{}", "module": "{}", "file": "{}", "line": {} }} ```"#,
                            now,
                            emoji,
                            visitor.output,
                            level,
                            module.unwrap_or(&self.unknown),
                            file.unwrap_or(&self.unknown).replace("\\", "/"),
                            line.unwrap_or(0)
                        );
                        (json, Some(teloxide::types::ParseMode::MarkdownV2))
                    }
                TelegramFormat::Template(tpl) => {
                    let tpl = tpl.replace("{emoji}", emoji)
                        .replace("{time}", &now)
                        .replace("{msg}", &visitor.output)
                        .replace("{level}", &level.to_string())
                        .replace("{module}", module.unwrap_or(&self.unknown))
                        .replace("{file}", &file.unwrap_or(&self.unknown).replace("\\", "/"))
                        .replace("{line}", &line.unwrap_or(0).to_string());
                    (tpl, None)
                }
                ,
        };

        let sender = self.sender.clone();
        tokio::spawn(async move {
            sender.send(msg, parse_mode).await;
        });
    }
}

/// -------------------- Builder --------------------
#[derive(Default)]
pub struct TelegramLayerBuilder {
    bot: Option<Arc<Bot>>,
    chat_ids: Option<Vec<i64>>,
    format: Option<TelegramFormat>,
    tag: Option<Vec<String>>,
    unknown: Option<String>,
}

impl TelegramLayerBuilder {

    pub fn bot(mut self, bot: Arc<Bot>) -> Self {
        self.bot = Some(bot);
        self
    }

    /// 配置单个通知用户ID
    pub fn chat_id(mut self, chat_id: i64) -> Self {
        self.chat_ids = Some(vec![chat_id]);
        self
    }

    /// 配置多个通知用户ID's
    pub fn chat_ids(mut self, chat_id: Vec<i64>) -> Self {
        self.chat_ids = Some(chat_id);
        self
    }
    /// markdown 格式发送
    pub fn markdown(mut self) -> Self {
        self.format = Some(TelegramFormat::Markdown);
        self
    }
    /// 普通文本发送
    pub fn text(mut self) -> Self {
        self.format = Some(TelegramFormat::Text);
        self
    }
    /// 按json格式发送
    pub fn json(mut self) -> Self {
        self.format = Some(TelegramFormat::Json);
        self
    }
    /// 模板字符串，使用占位符 {emoji} {time} {msg} {level} {module} {file} {line}
    pub fn template(mut self, tpl: &'static str) -> Self {
        self.format = Some(TelegramFormat::Template(tpl));
        self
    }
    // 自定义Tag标签，将包含TAG的消息发送到
    pub fn tag(mut self, tag: Vec<String>) -> Self {
        self.tag = Some(tag);
        self
    }
    //bot和chat_id
    pub fn with_bot(mut self, bot_token: String, chat_ids: Vec<i64>) -> Self {
        self.bot = Some(Arc::new(Bot::new(bot_token)));
        self.chat_ids = Some(chat_ids);
        self
    }
    // unknown 用于设置未知或解析失败设置的默认输出内容 ，默认为 Unknown
    pub fn unknown(mut self, unknown: String) -> Self {
        self.unknown = Some(unknown);
        self
    }

    pub fn build(self) -> TelegramLayer {
        let bot = self.bot.expect("Bot must be set");
        let chat_ids = self.chat_ids.expect("chat_id must be set");
        let format = self.format.unwrap_or(TelegramFormat::Text);
        let unknown = self.unknown.unwrap_or("Unknown".to_string());

        TelegramLayer {
            sender: TelegramSender::new(bot, chat_ids),
            format,
            tag: self.tag.unwrap_or(vec![]),
            unknown,
        }
    }
}
