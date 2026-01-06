//! 🎨 Beautiful logging system for PersonaForge

use nu_ansi_term::{Color, Style};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{self, FormatEvent, FormatFields};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

// ═══════════════════════════════════════════════════════════════════════════════
// 📊 METRICS & STATS
// ═══════════════════════════════════════════════════════════════════════════════

pub struct Metrics {
    pub messages_processed: AtomicU64,
    pub llm_requests: AtomicU64,
    pub llm_total_time_ms: AtomicU64,
    pub embeddings_generated: AtomicU64,
    pub embeddings_total_time_ms: AtomicU64,
    pub voice_transcriptions: AtomicU64,
    pub vision_analyses: AtomicU64,
    pub web_searches: AtomicU64,
    pub errors: AtomicU64,
    pub start_time: Instant,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            messages_processed: AtomicU64::new(0),
            llm_requests: AtomicU64::new(0),
            llm_total_time_ms: AtomicU64::new(0),
            embeddings_generated: AtomicU64::new(0),
            embeddings_total_time_ms: AtomicU64::new(0),
            voice_transcriptions: AtomicU64::new(0),
            vision_analyses: AtomicU64::new(0),
            web_searches: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    pub fn record_llm_request(&self, duration_ms: u64) {
        self.llm_requests.fetch_add(1, Ordering::Relaxed);
        self.llm_total_time_ms.fetch_add(duration_ms, Ordering::Relaxed);
    }

    pub fn record_embedding(&self, duration_ms: u64) {
        self.embeddings_generated.fetch_add(1, Ordering::Relaxed);
        self.embeddings_total_time_ms.fetch_add(duration_ms, Ordering::Relaxed);
    }

    pub fn avg_llm_time_ms(&self) -> u64 {
        let requests = self.llm_requests.load(Ordering::Relaxed);
        if requests == 0 { 0 } else { self.llm_total_time_ms.load(Ordering::Relaxed) / requests }
    }

    pub fn avg_embedding_time_ms(&self) -> u64 {
        let count = self.embeddings_generated.load(Ordering::Relaxed);
        if count == 0 { 0 } else { self.embeddings_total_time_ms.load(Ordering::Relaxed) / count }
    }

    pub fn format_uptime(&self) -> String {
        let secs = self.start_time.elapsed().as_secs();
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;
        if hours > 0 { format!("{}h {}m {}s", hours, mins, secs) }
        else if mins > 0 { format!("{}m {}s", mins, secs) }
        else { format!("{}s", secs) }
    }
}

impl Default for Metrics {
    fn default() -> Self { Self::new() }
}

pub static METRICS: Lazy<Metrics> = Lazy::new(Metrics::new);

// ═══════════════════════════════════════════════════════════════════════════════
// 🔇 SPAM SUPPRESSION
// ═══════════════════════════════════════════════════════════════════════════════

struct SpamTracker {
    last_messages: HashMap<String, (Instant, u32)>,
    suppression_window: Duration,
    max_repeats: u32,
}

impl SpamTracker {
    fn new() -> Self {
        Self {
            last_messages: HashMap::new(),
            suppression_window: Duration::from_secs(5),
            max_repeats: 3,
        }
    }

    fn should_log(&mut self, key: &str) -> Option<u32> {
        let now = Instant::now();
        self.last_messages.retain(|_, (time, _)| now.duration_since(*time) < self.suppression_window * 2);
        
        if let Some((last_time, count)) = self.last_messages.get_mut(key) {
            if now.duration_since(*last_time) < self.suppression_window {
                *count += 1;
                *last_time = now;
                if *count > self.max_repeats { return None; }
                return Some(*count);
            } else {
                *last_time = now;
                let old_count = *count;
                *count = 1;
                if old_count > self.max_repeats { return Some(old_count); }
            }
        } else {
            self.last_messages.insert(key.to_string(), (now, 1));
        }
        Some(1)
    }
}

static SPAM_TRACKER: Lazy<Mutex<SpamTracker>> = Lazy::new(|| Mutex::new(SpamTracker::new()));

// ═══════════════════════════════════════════════════════════════════════════════
// 🎨 CUSTOM FORMATTER
// ═══════════════════════════════════════════════════════════════════════════════

pub struct PrettyFormatter;

impl<S, N> FormatEvent<S, N> for PrettyFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &fmt::FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let now = chrono::Local::now();
        write!(writer, "{} ", Color::DarkGray.paint(now.format("%H:%M:%S").to_string()))?;

        let (emoji, level_style) = match *event.metadata().level() {
            Level::ERROR => ("❌", Style::new().fg(Color::Red).bold()),
            Level::WARN => ("⚠️ ", Style::new().fg(Color::Yellow)),
            Level::INFO => ("", Style::new().fg(Color::Green)),
            Level::DEBUG => ("🔍", Style::new().fg(Color::Blue)),
            Level::TRACE => ("📍", Style::new().fg(Color::Purple)),
        };

        let level_str = match *event.metadata().level() {
            Level::ERROR => "ERR",
            Level::WARN => "WRN",
            Level::INFO => "INF",
            Level::DEBUG => "DBG",
            Level::TRACE => "TRC",
        };

        write!(writer, "{}{} ", emoji, level_style.paint(level_str))?;

        let target = event.metadata().target();
        let short_target = if target.starts_with("persona_forge::") {
            &target[15..]
        } else if target.len() > 25 {
            &target[target.len() - 25..]
        } else {
            target
        };
        
        write!(writer, "{} ", Color::Cyan.paint(format!("[{}]", short_target)))?;
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🚀 INITIALIZATION
// ═══════════════════════════════════════════════════════════════════════════════

pub fn init() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new("info")
                .add_directive("persona_forge=debug".parse().unwrap())
                .add_directive("sqlx=warn".parse().unwrap())
                .add_directive("hyper=warn".parse().unwrap())
                .add_directive("reqwest=warn".parse().unwrap())
                .add_directive("teloxide=info".parse().unwrap())
        });

    let fmt_layer = fmt::layer()
        .event_format(PrettyFormatter)
        .with_ansi(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}

// ═══════════════════════════════════════════════════════════════════════════════
// 📝 STARTUP HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

pub fn print_banner() {
    let banner = r#"
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║   ██████╗ ███████╗██████╗ ███████╗ ██████╗ ███╗   ██╗ █████╗ ║
║   ██╔══██╗██╔════╝██╔══██╗██╔════╝██╔═══██╗████╗  ██║██╔══██╗║
║   ██████╔╝█████╗  ██████╔╝███████╗██║   ██║██╔██╗ ██║███████║║
║   ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║██║   ██║██║╚██╗██║██╔══██║║
║   ██║     ███████╗██║  ██║███████║╚██████╔╝██║ ╚████║██║  ██║║
║   ╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝  ╚═══╝╚═╝  ╚═╝║
║                                                              ║
║              🤖 F O R G E   v1.0.0                           ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝"#;
    
    println!("{}", Color::Cyan.bold().paint(banner));
    println!();
}

pub fn print_config(bot_name: &str, owner_id: u64, llm_model: &str, vision: bool, voice: bool, web_search: bool) {
    let check = Color::Green.paint("✓");
    let cross = Color::Red.paint("✗");
    
    println!("{}", Color::White.bold().paint("┌─ Configuration ─────────────────────────────────────────────┐"));
    println!("│  {}  Bot Name    │ {:<40} │", Color::Blue.paint("🤖"), Color::White.bold().paint(bot_name));
    println!("│  {}  Owner ID    │ {:<40} │", Color::Yellow.paint("👤"), owner_id);
    println!("│  {}  LLM Model   │ {:<40} │", Color::Magenta.paint("🧠"), Color::Cyan.paint(llm_model));
    println!("│  {}  Vision      │ {:<40} │", if vision { &check } else { &cross }, if vision { "Enabled" } else { "Disabled" });
    println!("│  {}  Voice       │ {:<40} │", if voice { &check } else { &cross }, if voice { "Enabled" } else { "Disabled" });
    println!("│  {}  Web Search  │ {:<40} │", if web_search { &check } else { &cross }, if web_search { "Enabled" } else { "Disabled" });
    println!("{}", Color::White.bold().paint("└──────────────────────────────────────────────────────────────┘"));
    println!();
}

pub fn print_db_connected(url: &str) {
    let display_url = if url.len() > 50 { format!("...{}", &url[url.len()-47..]) } else { url.to_string() };
    println!("{}  Database connected: {}", Color::Green.paint("✓"), Color::Cyan.paint(display_url));
}

pub fn print_bot_info(name: &str, username: &str) {
    println!("{}  Bot identity: {} {}", Color::Green.paint("✓"), Color::White.bold().paint(name), Color::DarkGray.paint(format!("(@{})", username)));
}

pub fn print_webapp_started(port: u16) {
    println!("{}  WebApp listening on port {}", Color::Green.paint("✓"), Color::Cyan.bold().paint(port.to_string()));
}

pub fn print_ready() {
    println!();
    println!("{}", Color::Green.bold().paint("══════════════════════════════════════════════════════════════"));
    println!("{}  {}", Color::Green.bold().paint("🚀"), Color::Green.bold().paint("PersonaForge is ready and listening!"));
    println!("{}", Color::Green.bold().paint("══════════════════════════════════════════════════════════════"));
    println!();
}

pub fn print_shutdown() {
    let m = &*METRICS;
    println!();
    println!("{}", Color::Yellow.bold().paint("══════════════════════════════════════════════════════════════"));
    println!("{}  Shutting down PersonaForge...", Color::Yellow.paint("👋"));
    println!();
    println!("{}  Session Statistics:", Color::White.bold().paint("📊"));
    println!("   ├─ Uptime: {}", Color::Cyan.paint(m.format_uptime()));
    println!("   ├─ Messages: {}", m.messages_processed.load(Ordering::Relaxed));
    println!("   ├─ LLM: {} req (avg {}ms)", m.llm_requests.load(Ordering::Relaxed), m.avg_llm_time_ms());
    println!("   ├─ Embeddings: {} (avg {}ms)", m.embeddings_generated.load(Ordering::Relaxed), m.avg_embedding_time_ms());
    println!("   ├─ Voice: {}", m.voice_transcriptions.load(Ordering::Relaxed));
    println!("   ├─ Vision: {}", m.vision_analyses.load(Ordering::Relaxed));
    println!("   └─ Errors: {}", m.errors.load(Ordering::Relaxed));
    println!("{}", Color::Yellow.bold().paint("══════════════════════════════════════════════════════════════"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 📊 STRUCTURED EVENT LOGGING
// ═══════════════════════════════════════════════════════════════════════════════

pub fn log_message_received(chat_id: i64, user: &str, text_preview: &str, has_media: bool) {
    METRICS.messages_processed.fetch_add(1, Ordering::Relaxed);
    let media_badge = if has_media { " 📎" } else { "" };
    let preview = if text_preview.len() > 50 { format!("{}...", &text_preview[..47]) } else { text_preview.to_string() };
    tracing::info!(target: "messages", "💬 {} in {}{}: \"{}\"", Color::Cyan.paint(user), Color::Yellow.paint(chat_id.to_string()), media_badge, Color::White.paint(preview));
}

pub fn log_llm_request(model: &str, prompt_len: usize) {
    tracing::debug!(target: "llm", "🧠 Request to {} ({} chars)", Color::Magenta.paint(model), prompt_len);
}

pub fn log_llm_response(duration_ms: u64, response_len: usize) {
    METRICS.record_llm_request(duration_ms);
    let time_color = if duration_ms < 1000 { Color::Green } else if duration_ms < 3000 { Color::Yellow } else { Color::Red };
    tracing::info!(target: "llm", "🧠 Response in {} ({} chars)", time_color.paint(format!("{}ms", duration_ms)), response_len);
}

pub fn log_embedding(duration_ms: u64) {
    METRICS.record_embedding(duration_ms);
    let key = "embedding";
    let mut tracker = SPAM_TRACKER.lock().unwrap();
    match tracker.should_log(key) {
        Some(count) if count == 1 => { tracing::debug!(target: "llm", "📐 Embedding in {}ms", duration_ms); }
        Some(count) if count > 3 => {
            let total = METRICS.embeddings_generated.load(Ordering::Relaxed);
            let avg = METRICS.avg_embedding_time_ms();
            tracing::debug!(target: "llm", "📐 Embeddings: {} total (avg {}ms)", total, avg);
        }
        _ => {}
    }
}

pub fn log_voice_transcription(duration_ms: u64, text_preview: &str) {
    METRICS.voice_transcriptions.fetch_add(1, Ordering::Relaxed);
    let preview = if text_preview.len() > 40 { format!("{}...", &text_preview[..37]) } else { text_preview.to_string() };
    tracing::info!(target: "voice", "🎤 Transcribed in {}ms: \"{}\"", duration_ms, Color::White.paint(preview));
}

pub fn log_vision_analysis(duration_ms: u64, frame_count: usize) {
    METRICS.vision_analyses.fetch_add(1, Ordering::Relaxed);
    tracing::info!(target: "vision", "👁️  Analyzed {} frames in {}ms", frame_count, duration_ms);
}

pub fn log_web_search(query: &str, result_count: usize) {
    METRICS.web_searches.fetch_add(1, Ordering::Relaxed);
    let query_preview = if query.len() > 30 { format!("{}...", &query[..27]) } else { query.to_string() };
    tracing::info!(target: "web", "🌐 Search \"{}\" → {} results", Color::Cyan.paint(query_preview), result_count);
}

pub fn log_error(context: &str, error: &str) {
    METRICS.errors.fetch_add(1, Ordering::Relaxed);
    tracing::error!(target: "error", "{}: {}", Color::Red.bold().paint(context), error);
}

pub fn log_api_request(method: &str, path: &str, status: u16) {
    let status_color = if status < 300 { Color::Green } else if status < 400 { Color::Yellow } else { Color::Red };
    tracing::debug!(target: "api", "🌐 {} {} → {}", Color::Cyan.paint(method), path, status_color.paint(status.to_string()));
}
