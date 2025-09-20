use chrono::Local;
use tracing_subscriber::Layer;
use tracing_telegram::TelegramLayer;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use rand::{Rng, SeedableRng};
use anyhow::Result;
use futures::future::join_all;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let bot_token = std::env::var("BOT_TOKEN").expect("BOT_TOKEN must be set");
    let chat_id: i64 = std::env::var("CHAT_ID")
        .expect("CHAT_ID must be set")
        .parse()
        .expect("CHAT_ID must be a valid integer");

    let bot = Arc::new(Bot::new(bot_token));

    // 控制台日志层
    let console_layer = fmt::layer()
        .compact()
        .with_timer(LocalTime::rfc_3339())
        .with_filter(LevelFilter::INFO);

    // Telegram 日志层
    let telegram_layer = TelegramLayer::builder()
        .bot(bot.clone())
        .chat_id(chat_id)
        .markdown()
        .build()
        .with_filter(LevelFilter::INFO);

    // 初始化日志订阅器
    tracing_subscriber::registry()
        .with(console_layer)
        .with(telegram_layer)
        .init();

    info!("🚀 服务启动成功！开始随机发送测试日志...");

    // 总测试时间
    let test_duration = Duration::from_secs(300000); // 5分钟
    let end_time = Local::now() + chrono::Duration::from_std(test_duration)?;

    // 并发运行所有模拟任务
    let scenarios = vec![
        tokio::spawn(simulate_web_server_random(end_time)),
        tokio::spawn(simulate_database_operations_random(end_time)),
        tokio::spawn(simulate_user_activity_random(end_time)),
        tokio::spawn(simulate_system_monitoring_random(end_time)),
        tokio::spawn(simulate_error_scenarios_random(end_time)),
    ];

    join_all(scenarios).await;

    info!("✅ 随机日志测试完成！");
    Ok(())
}

/// ---------------- 随机日志模拟函数 ----------------

async fn simulate_web_server_random(end_time: chrono::DateTime<Local>) {
    let mut rng =rand::rngs::StdRng::from_rng(&mut rand::rng()); // 使用系统随机数生成器
    let endpoints = vec!["/api/v1/users","/api/v1/products","/api/v1/orders","/api/v1/auth","/health","/metrics"];
    let methods = vec!["GET","POST","PUT","DELETE","PATCH"];
    let status_codes = vec![200,201,400,401,403,404,500];

    loop {
        if Local::now() >= end_time { break; }

        let sleep_time = Duration::from_secs(rng.random_range(15..30)); // 延长延时
        sleep(sleep_time).await;

        let endpoint = endpoints[rng.random_range(0..endpoints.len())];
        let method = methods[rng.random_range(0..methods.len())];
        let status = status_codes[rng.random_range(0..status_codes.len())];
        let response_time = rng.random_range(50..500);

        let roll: f64 = rng.random();
        if status >= 400 || roll > 0.85 {
            error!("❌ HTTP {} {} -> Status: {} ({}ms)", method, endpoint, status, response_time);
        } else if roll > 0.6 {
            warn!("⚠️ HTTP {} {} -> Status: {} ({}ms)", method, endpoint, status, response_time);
        } else {
            info!("🌐 HTTP {} {} -> Status: {} ({}ms)", method, endpoint, status, response_time);
        }
    }
}

async fn simulate_database_operations_random(end_time: chrono::DateTime<Local>) {
     let mut rng =rand::rngs::StdRng::from_rng(&mut rand::rng()); // 使用系统随机数生成器
    let operations = vec!["SELECT * FROM users","INSERT INTO orders","UPDATE products SET","DELETE FROM logs","CREATE TABLE temp","DROP TABLE old_data"];
    let tables = vec!["users","products","orders","logs","settings"];

    loop {
        if Local::now() >= end_time { break; }

     let sleep_time = Duration::from_secs(rng.random_range(30..50)); // 延长延时
        sleep(sleep_time).await;

        let operation = operations[rng.random_range(0..operations.len())];
        let table = tables[rng.random_range(0..tables.len())];
        let duration = rng.random_range(10..200);
        let rows_affected = rng.random_range(1..1000);

        let roll: f64 = rng.random();
        if roll > 0.95 {
            error!("💥 数据库错误: 执行 {} {} 时连接超时", operation, table);
        } else if roll > 0.85 {
            warn!("🐌 慢查询警告: {} {} 耗时 {}ms", operation, table, duration + 500);
        } else {
            info!("💾 数据库操作: {} {} -> 耗时: {}ms, 影响行数: {}", operation, table, duration, rows_affected);
        }
    }
}

async fn simulate_user_activity_random(end_time: chrono::DateTime<Local>) {
     let mut rng =rand::rngs::StdRng::from_rng(&mut rand::rng()); // 使用系统随机数生成器
    let actions = vec!["用户登录","用户注册","密码重置","资料更新","订单创建","支付完成","商品浏览","购物车操作"];
    let users = vec!["alice","bob","charlie","diana","eve","frank"];

    loop {
        if Local::now() >= end_time { break; }

        let sleep_time = Duration::from_secs(rng.random_range(1..6));
        sleep(sleep_time).await;

        let action = actions[rng.random_range(0..actions.len())];
        let user = users[rng.random_range(0..users.len())];
        let roll: f64 = rng.random();

        if roll > 0.95 {
            warn!("👤 {}: {} 失败", user, action);
        } else {
            info!("👤 {}: {} 成功", user, action);
        }

        if rng.random_bool(0.1) {
            let session_duration = rng.random_range(300..3600);
            let pages_visited = rng.random_range(5..50);
            info!("📊 用户分析: {} 会话时长: {}秒, 浏览页面: {}", user, session_duration, pages_visited);
        }
    }
}

async fn simulate_system_monitoring_random(end_time: chrono::DateTime<Local>) {
     let mut rng =rand::rngs::StdRng::from_rng(&mut rand::rng()); // 使用系统随机数生成器

    loop {
        if Local::now() >= end_time { break; }

       let sleep_time = Duration::from_secs(rng.random_range(50..60)); // 延长延时
        sleep(sleep_time).await;

        let cpu = rng.random_range(10..90);
        let mem = rng.random_range(20..85);
        let disk = rng.random_range(30..95);
        let net = rng.random_range(100..1000);

        if cpu > 80 { warn!("🖥️ CPU使用率过高: {}%", cpu); } else { info!("🖥️ CPU使用率: {}%", cpu); }
        if mem > 75 { warn!("💾 内存使用率警告: {}%", mem); } else { info!("💾 内存使用率: {}%", mem); }
        if disk > 90 { error!("💿 磁盘空间不足: {}% 已使用", disk); } else { debug!("💿 磁盘使用率: {}%", disk); }
        info!("🌐 网络流量: {} MB/s", net);

        if rng.random_bool(0.03) {
            let events = vec!["系统备份完成","安全扫描开始","证书即将过期","新版本可用","配置更新应用"];
            let event = events[rng.random_range(0..events.len())];
            info!("🔔 系统事件: {}", event);
        }
    }
}

async fn simulate_error_scenarios_random(end_time: chrono::DateTime<Local>) {
    let mut rng =rand::rngs::StdRng::from_rng(&mut rand::rng()); // 使用系统随机数生成器
    let errors = vec!["连接超时","权限拒绝","资源不足","数据校验失败","第三方服务不可用","缓存击穿","死锁检测"];
    let services = vec!["用户服务","订单服务","支付服务","库存服务","消息队列","缓存服务","数据库"];

    loop {
        if Local::now() >= end_time { break; }

        let sleep_time = Duration::from_secs(rng.random_range(180..300)); // 延长延时
        sleep(sleep_time).await;

        let error_type = errors[rng.random_range(0..errors.len())];
        let service = services[rng.random_range(0..services.len())];
        let roll: f64 = rng.random();

        if roll > 0.7 {
            error!("💥 严重错误: {} -> {}, 影响用户: {}", service, error_type, rng.random_range(100..10000));
            if rng.random_bool(0.2) {
                error!("🔍 堆栈跟踪:\n\tat com.example.Service.handleRequest(Service.java:123)\n\tat com.example.Controller.process(Controller.java:456)\n\tat java.base/java.lang.Thread.run(Thread.java:834)");
            }
        } else {
            warn!("⚠️ 警告: {} -> {}", service, error_type);
        }
    }
}
