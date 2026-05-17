use std::sync::{Arc, Mutex};
use tracing::{field::Visit, Level, Subscriber};
use tracing_subscriber::Layer;

pub struct EguiLoggerLayer {
    logs: Arc<Mutex<Vec<String>>>,
}

impl EguiLoggerLayer {
    pub fn new(logs: Arc<Mutex<Vec<String>>>) -> Self {
        Self { logs }
    }
}

impl<S: Subscriber> Layer<S> for EguiLoggerLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = LogVisitor::new();
        event.record(&mut visitor);

        // 核心修改：只有标记了 user = true 的日志才会显示在 UI 上
        if !visitor.is_user_facing {
            return;
        }

        let level = *event.metadata().level();
        let prefix = match level {
            Level::ERROR => "❌ ",
            Level::WARN => "⚠️ ",
            Level::INFO => "",
            Level::DEBUG => "🔍 ",
            Level::TRACE => "📝 ",
        };

        let now = chrono::Local::now().format("%H:%M:%S").to_string();
        let msg = format!("[{}] {}{}", now, prefix, visitor.message);
        crate::leetcode::automation::daily::add_log(&self.logs, &msg);
    }
}

struct LogVisitor {
    message: String,
    is_user_facing: bool,
}

impl LogVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
            is_user_facing: false,
        }
    }
}

impl Visit for LogVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else if field.name() == "user" {
            if let Ok(val) = format!("{:?}", value).parse::<bool>() {
                self.is_user_facing = val;
            }
        }
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        if field.name() == "user" {
            self.is_user_facing = value;
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }
}

/// 初始化全局 tracing 订阅者，将技术日志输出到文件，关键日志输出到 UI
pub fn init_tracing(logs: Arc<Mutex<Vec<String>>>) {
    use tracing_subscriber::prelude::*;
    
    // 1. 配置文件日志（技术日志，全量 DEBUG 保存）
    let log_dir = if let Some(mut path) = dirs::home_dir() {
        path.push(".leetcode_automation");
        path.push("logs");
        path
    } else {
        std::path::PathBuf::from(".logs")
    };
    
    let file_appender = tracing_appender::rolling::daily(log_dir, "automation.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    Box::leak(Box::new(_guard));

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_filter(tracing_subscriber::filter::LevelFilter::DEBUG);

    // 2. 配置 UI 日志（用户日志，仅关键信息）
    let egui_layer = EguiLoggerLayer::new(logs);

    // 3. 配置终端输出（开发调试用，保持 INFO 级别）
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_level(true)
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .with(egui_layer)
        .init();
}
