use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::mpsc;
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;
use tracing::field::{Field, Visit};
use chrono::Local;
pub use teloxide;
pub use tracing_subscriber;


/// 日志输出格式
#[derive(Clone, Copy)]
pub enum TelegramFormat {
    Text,
    Markdown,
    Json
}   

/// 队列异步发送器
#[derive(Clone)]
struct TelegramSender {
    sender: mpsc::Sender<(String, Option<teloxide::types::ParseMode>)>,
}

impl TelegramSender {
    pub fn new(bot: Arc<Bot>, chat_id: i64) -> Self {
    //   let (ftx,frx ) = futures::channel::mpsc::channel::<(String,Option<teloxide::types::ParseMode>)>(100);
        let (tx, mut rx) = mpsc::channel::<(String, Option<teloxide::types::ParseMode>)>(100);
        let bot_clone = bot.clone();

        tokio::spawn(async move {
            while let Some((msg, parse_mode)) = rx.recv().await {
                let mut req = bot_clone.send_message(ChatId(chat_id), msg);
                if let Some(pm) = parse_mode {
                    req = req.parse_mode(pm);
                }
                if let Err(err) = req.await {
                    eprintln!("Failed to send log to Telegram: {}", err);
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
    tag:Vec<String>,
}

impl TelegramLayer {
  pub   fn new(bot: Arc<Bot>, chat_id: i64, format: TelegramFormat) -> Self {
        Self {
            sender: TelegramSender::new(bot, chat_id),
            format,
            tag: Vec::new(),
        }
    }
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

impl<S> Layer<S> for TelegramLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = MessageVisitor { output: String::new() };
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
        
        let meta  = event.metadata();
        let line  = meta.line();
        let file = meta.file();
        let module = meta.module_path();
        let level = meta.level();
        // info!("{} {}:{} {}", level, module.unwrap_or("unknown"), line.unwrap_or(0), file.unwrap_or("unknown").replace("\\", "/"));
    
        // 当前本地时间
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        // 根据格式生成最终消息
        let (msg, parse_mode) = match self.format {
            TelegramFormat::Text => (format!("[{}] {}", now, visitor.output), None),
            TelegramFormat::Markdown => (
                // format!("```\n[{}] {} \n```", now, visitor.output),
                format!("```\n[{}] {} {}:{} {} {}\n```", now, level, module.unwrap_or("unknown"), line.unwrap_or(0), file.unwrap_or("unknown").replace("\\", "/"), visitor.output),
                Some(teloxide::types::ParseMode::MarkdownV2),
            ),
            TelegramFormat::Json => {
                let json_msg = format!("```json\n{{\"time\": \"{}\", \"msg\": \"{}\"}}\n```", now, visitor.output);
                (json_msg, Some(teloxide::types::ParseMode::MarkdownV2))
            }
        };

        let sender = self.sender.clone();
        tokio::spawn(async move {
            sender.send(msg, parse_mode).await;
        });
    }
}

/// 转义 MarkdownV2 特殊字符
pub fn escape_markdown_v2(text: &str) -> String {
    let chars_to_escape = r"_*[]()~`>#+-=|{}.!";
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        if chars_to_escape.contains(c) {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

/// -------------------- Builder --------------------
#[derive(Default)]
pub struct TelegramLayerBuilder {
    bot: Option<Arc<Bot>>,
    chat_id: Option<i64>,
    format: Option<TelegramFormat>,
    tag:Option<Vec<String>>    
}

impl TelegramLayerBuilder {
    pub fn bot(mut self, bot: Arc<Bot>) -> Self {
        self.bot = Some(bot);
        self
    }

    pub fn chat_id(mut self, chat_id: i64) -> Self {
        self.chat_id = Some(chat_id);
        self
    }
    pub  fn markdown(mut self)-> Self {
        self.format = Some(TelegramFormat::Markdown);
        self
    }
    pub  fn text(mut self)-> Self {
        self.format = Some(TelegramFormat::Text);
        self
    }
    pub  fn json(mut self)-> Self {
        self.format = Some(TelegramFormat::Json);
        self
    }
    // 用于设置日志标签，用于过滤日志-
    pub  fn tag(mut self, tag:Vec<String>)-> Self {
        self.tag = Some(tag);
        self
    }
    //注意这里用于通过内部构建持有bot和chat_id
    pub fn  with_bot(mut self,bot_token:String,chat_id: i64) -> Self{
        self.bot = Some(Arc::new(Bot::new(bot_token)));
        self.chat_id = Some(chat_id);
       self
    }

    pub fn build(self) -> TelegramLayer {
        let bot = self.bot.expect("Bot must be set");
        let chat_id = self.chat_id.expect("chat_id must be set");
        let format = self.format.unwrap_or(TelegramFormat::Text);

        TelegramLayer {
            sender: TelegramSender::new(bot, chat_id),
            format,
            tag: self.tag.unwrap_or(vec![]),
        }
    }
}