mod services;
mod models;

use actix_web::{web, App, HttpServer, HttpResponse};
use actix_files as fs;
use tera::Tera;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashSet;

use models::config::AppConfig;
use services::health::ServiceChecker;

pub struct AppState {
    pub tera: Tera,
    pub config: AppConfig,
    pub checker: Arc<RwLock<ServiceChecker>>,
}

async fn index(data: web::Data<AppState>) -> HttpResponse {
    let checker = data.checker.read().await;
    let statuses = checker.get_statuses();
    let online_count = statuses.iter().filter(|s| s.status == "up").count();

    let vitals = services::system::get_vitals().await;

    let mut ctx = tera::Context::new();
    ctx.insert("services", &statuses);
    ctx.insert("online_count", &online_count);
    ctx.insert("total_count", &statuses.len());
    ctx.insert("vitals", &vitals);

    // Greeting based on time of day
    let hour = chrono::Local::now().format("%H").to_string().parse::<u32>().unwrap_or(12);
    let greeting = match hour {
        5..=11 => "Good morning",
        12..=17 => "Good afternoon",
        18..=22 => "Good evening",
        _ => "Good night",
    };
    let user_name = std::env::var("DASHBOARD_USER").unwrap_or_else(|_| "Admin".into());
    ctx.insert("greeting", greeting);
    ctx.insert("user_name", &user_name);
    ctx.insert("time", &chrono::Local::now().format("%H:%M").to_string());
    ctx.insert("date", &chrono::Local::now().format("%a %d %b").to_string());

    match data.tera.render("index.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

/// HTMX partial: refreshes service status tiles
async fn htmx_services(data: web::Data<AppState>) -> HttpResponse {
    let checker = data.checker.read().await;
    let statuses = checker.get_statuses();
    let online_count = statuses.iter().filter(|s| s.status == "up").count();
    let mut ctx = tera::Context::new();
    ctx.insert("services", &statuses);
    ctx.insert("online_count", &online_count);
    ctx.insert("total_count", &statuses.len());

    match data.tera.render("partials/services.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Partial error: {}", e)),
    }
}

/// HTMX partial: system vitals with ring gauges
async fn htmx_vitals(data: web::Data<AppState>) -> HttpResponse {
    let vitals = services::system::get_vitals().await;
    let mut ctx = tera::Context::new();
    ctx.insert("vitals", &vitals);

    match data.tera.render("partials/vitals.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Partial error: {}", e)),
    }
}

/// HTMX partial: downloads from qBittorrent
async fn htmx_downloads(data: web::Data<AppState>) -> HttpResponse {
    let downloads = services::downloads::get_downloads().await;
    let mut ctx = tera::Context::new();
    ctx.insert("downloads", &downloads);

    match data.tera.render("partials/downloads.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Partial error: {}", e)),
    }
}

/// HTMX partial: weather info
async fn htmx_weather(data: web::Data<AppState>) -> HttpResponse {
    let weather = services::weather::get_weather().await;
    let mut ctx = tera::Context::new();
    ctx.insert("weather", &weather);

    match data.tera.render("partials/weather.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Partial error: {}", e)),
    }
}

/// HTMX partial: storage info
async fn htmx_storage(data: web::Data<AppState>) -> HttpResponse {
    let vitals = services::system::get_vitals().await;
    let mut ctx = tera::Context::new();
    ctx.insert("vitals", &vitals);

    match data.tera.render("partials/storage.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Partial error: {}", e)),
    }
}

/// HTMX partial: live date/time
async fn htmx_datetime(data: web::Data<AppState>) -> HttpResponse {
    let mut ctx = tera::Context::new();
    let hour = chrono::Local::now().format("%H").to_string().parse::<u32>().unwrap_or(12);
    let greeting = match hour {
        5..=11 => "Good morning",
        12..=17 => "Good afternoon",
        18..=22 => "Good evening",
        _ => "Good night",
    };
    let user_name = std::env::var("DASHBOARD_USER").unwrap_or_else(|_| "Admin".into());
    ctx.insert("greeting", greeting);
    ctx.insert("user_name", &user_name);
    ctx.insert("time", &chrono::Local::now().format("%H:%M").to_string());
    ctx.insert("date", &chrono::Local::now().format("%a %d %b").to_string());

    match data.tera.render("partials/datetime.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Partial error: {}", e)),
    }
}

/// Structured log entry for the template
#[derive(serde::Serialize)]
struct LogEntry {
    time: String,
    level: String,
    level_lower: String,
    subsystem: String,
    message: String,
}

/// Parse a single NDJSON log line into a structured entry
fn parse_log_line(line: &str) -> Option<LogEntry> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    let meta = v.get("_meta")?;

    // Time — parse ISO and convert to local HH:MM:SS
    let time_str = v.get("time")
        .and_then(|t| t.as_str())
        .unwrap_or("");
    let time = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(time_str) {
        dt.with_timezone(&chrono::Local).format("%H:%M:%S").to_string()
    } else {
        "??:??:??".to_string()
    };

    // Level
    let level = meta.get("logLevelName")
        .and_then(|l| l.as_str())
        .unwrap_or("INFO")
        .to_string();
    let level_lower = level.to_lowercase();

    // Subsystem — field "0" often looks like {"subsystem":"memory"}
    let subsystem_raw = v.get("0").unwrap_or(&serde_json::Value::Null);
    let subsystem = if let Some(s) = subsystem_raw.as_str() {
        // Try to parse as JSON to extract subsystem name
        if let Ok(sv) = serde_json::from_str::<serde_json::Value>(s) {
            sv.get("subsystem")
                .and_then(|ss| ss.as_str())
                .unwrap_or(s)
                .to_string()
        } else {
            s.to_string()
        }
    } else {
        String::new()
    };

    // Message — field "1" (main) + optional "2" (secondary)
    let msg1 = v.get("1").map(|m| {
        if let Some(s) = m.as_str() {
            s.to_string()
        } else {
            // For objects, extract key info compactly
            serde_json::to_string(m).unwrap_or_default()
        }
    }).unwrap_or_default();

    let msg2 = v.get("2").and_then(|m| {
        if let Some(s) = m.as_str() {
            Some(s.to_string())
        } else {
            None
        }
    });

    let message = if let Some(m2) = msg2 {
        format!("{} — {}", m2, msg1)
    } else {
        msg1
    };

    // HTML-escape the message
    let message = message
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    Some(LogEntry { time, level, level_lower, subsystem, message })
}

/// HTMX partial: clawdbot live logs
async fn htmx_clawdbot_logs(data: web::Data<AppState>) -> HttpResponse {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let log_path = format!("/tmp/clawdbot/clawdbot-{}.log", today);

    let entries: Vec<LogEntry> = match tokio::fs::read_to_string(&log_path).await {
        Ok(text) => {
            let lines: Vec<&str> = text.lines().collect();
            let start = if lines.len() > 80 { lines.len() - 80 } else { 0 };
            lines[start..].iter()
                .filter_map(|line| parse_log_line(line))
                .collect()
        }
        Err(_) => Vec::new(),
    };

    let has_logs = !entries.is_empty();

    let mut ctx = tera::Context::new();
    ctx.insert("entries", &entries);
    ctx.insert("has_logs", &has_logs);

    match data.tera.render("partials/clawdbot-logs.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}

/// HTMX partial: container logs in modal
async fn htmx_logs(data: web::Data<AppState>, path: web::Path<String>) -> HttpResponse {
    let container = path.into_inner();

    // Allowlist: only containers we know about
    let allowed: HashSet<String> = data.config.services.iter()
        .map(|s| s.container.clone()).collect();
    if !allowed.contains(&container) {
        return HttpResponse::BadRequest().body("Unknown container");
    }

    let output = tokio::process::Command::new("docker")
        .args(["logs", "--tail", "150", &container])
        .output().await;

    let logs = match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            let combined = if stdout.is_empty() { stderr.to_string() } else { format!("{}{}", stdout, stderr) };
            if combined.is_empty() { "No logs available.".to_string() } else { combined }
        }
        Err(e) => format!("Failed to fetch logs: {}", e),
    };

    // Escape HTML
    let escaped = logs
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    let html = format!(
        r#"<div class="flex items-center justify-between mb-4">
    <h2 class="text-base font-semibold text-white">{} — Logs</h2>
    <button onclick="document.getElementById('log-modal').classList.add('hidden')"
            class="p-1.5 rounded-lg hover:bg-nexus-tile transition-colors">
        <i data-lucide="x" class="w-4 h-4 text-nexus-muted"></i>
    </button>
</div>
<pre class="text-[11px] font-mono text-gray-300 bg-nexus-tile border border-nexus-tile-border rounded-tile p-4 overflow-auto max-h-[60vh] whitespace-pre-wrap break-all leading-relaxed">{}</pre>"#,
        container, escaped
    );

    HttpResponse::Ok().content_type("text/html").body(html)
}

/// HTMX partial: activity feed
async fn htmx_activity(data: web::Data<AppState>) -> HttpResponse {
    let checker = data.checker.read().await;
    let statuses = checker.get_statuses();
    let mut ctx = tera::Context::new();
    ctx.insert("services", &statuses);
    ctx.insert("now", &chrono::Local::now().format("%H:%M:%S").to_string());

    match data.tera.render("partials/activity.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Partial error: {}", e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
    println!("🚀 Nexus starting on http://0.0.0.0:{}", port);

    let config = AppConfig::load();
    let checker = Arc::new(RwLock::new(ServiceChecker::new(&config.services)));

    // Background health check loop
    let checker_bg = checker.clone();
    let services_bg = config.services.clone();
    tokio::spawn(async move {
        loop {
            {
                let mut c = checker_bg.write().await;
                c.check_all(&services_bg).await;
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    });

    let tera = Tera::new("templates/**/*.html").expect("Failed to load templates");

    let data = web::Data::new(AppState {
        tera,
        config,
        checker,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .route("/", web::get().to(index))
            .route("/htmx/services", web::get().to(htmx_services))
            .route("/htmx/vitals", web::get().to(htmx_vitals))
            .route("/htmx/downloads", web::get().to(htmx_downloads))
            .route("/htmx/weather", web::get().to(htmx_weather))
            .route("/htmx/storage", web::get().to(htmx_storage))
            .route("/htmx/datetime", web::get().to(htmx_datetime))
            .route("/htmx/activity", web::get().to(htmx_activity))
            .route("/htmx/logs/{container}", web::get().to(htmx_logs))
            .route("/htmx/clawdbot-logs", web::get().to(htmx_clawdbot_logs))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
