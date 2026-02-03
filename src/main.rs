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

/// HTMX partial: OpenClaw live logs
async fn htmx_openclaw_logs(data: web::Data<AppState>) -> HttpResponse {
    // OpenClaw logs at ~/.openclaw/logs/gateway.log (single file, not date-rotated)
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    let log_path = format!("{}/.openclaw/logs/gateway.log", home);

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

    match data.tera.render("partials/openclaw-logs.html", &ctx) {
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

// ─── New HTMX partials ──────────────────────────────────────────────────────

/// HTMX partial: media library stats (Jellyfin)
async fn htmx_media_stats(data: web::Data<AppState>) -> HttpResponse {
    let stats = services::jellyfin::get_media_stats().await;
    let mut ctx = tera::Context::new();
    ctx.insert("stats", &stats);

    match data.tera.render("partials/media-stats.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Partial error: {}", e)),
    }
}

/// HTMX partial: Sonarr/Radarr queue
async fn htmx_media_queue(data: web::Data<AppState>) -> HttpResponse {
    let queue = services::arr::get_media_queue().await;
    let mut ctx = tera::Context::new();
    ctx.insert("queue", &queue);

    match data.tera.render("partials/media-queue.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Partial error: {}", e)),
    }
}

/// HTMX partial: upcoming releases
async fn htmx_upcoming(data: web::Data<AppState>) -> HttpResponse {
    let upcoming = services::arr::get_upcoming().await;
    let mut ctx = tera::Context::new();
    ctx.insert("upcoming", &upcoming);

    match data.tera.render("partials/upcoming.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Partial error: {}", e)),
    }
}

/// HTMX partial: speedtest results
async fn htmx_speedtest(data: web::Data<AppState>) -> HttpResponse {
    let speedtest = services::speedtest::get_speedtest().await;
    let mut ctx = tera::Context::new();
    ctx.insert("speedtest", &speedtest);

    match data.tera.render("partials/speedtest.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Partial error: {}", e)),
    }
}

/// HTMX partial: container resource usage
async fn htmx_container_resources(data: web::Data<AppState>) -> HttpResponse {
    let resources = services::containers::get_container_resources().await;
    let mut ctx = tera::Context::new();
    ctx.insert("resources", &resources);

    match data.tera.render("partials/container-resources.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Partial error: {}", e)),
    }
}

/// HTMX partial: recently added to Jellyfin
async fn htmx_recently_added(data: web::Data<AppState>) -> HttpResponse {
    let recent = services::jellyfin::get_recently_added().await;
    let mut ctx = tera::Context::new();
    ctx.insert("recent", &recent);

    match data.tera.render("partials/recently-added.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Partial error: {}", e)),
    }
}

/// HTMX partial: system journal errors
async fn htmx_journal(data: web::Data<AppState>) -> HttpResponse {
    let journal = services::journal::get_journal().await;
    let mut ctx = tera::Context::new();
    ctx.insert("journal", &journal);

    match data.tera.render("partials/journal.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Partial error: {}", e)),
    }
}

// ─── Quick Actions ──────────────────────────────────────────────────────────

/// Allowed containers for restart
const RESTART_ALLOWLIST: &[&str] = &[
    "qbittorrent", "sonarr", "radarr", "bazarr", "prowlarr",
    "jellyfin", "jellyseerr", "flaresolverr", "homeassistant",
];

/// POST /api/actions/restart/{container}
async fn action_restart_container(path: web::Path<String>) -> HttpResponse {
    let container = path.into_inner();

    if !RESTART_ALLOWLIST.contains(&container.as_str()) {
        return HttpResponse::BadRequest().body(
            r#"<span class="text-nexus-error">Container not in allowlist</span>"#,
        );
    }

    let output = tokio::process::Command::new("docker")
        .args(["restart", &container])
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => HttpResponse::Ok().body(format!(
            r#"<span class="text-nexus-accent">✓ {} restarted</span>"#,
            container
        )),
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr);
            HttpResponse::Ok().body(format!(
                r#"<span class="text-nexus-error">✗ {}</span>"#,
                err.chars().take(80).collect::<String>()
            ))
        }
        Err(e) => HttpResponse::Ok().body(format!(
            r#"<span class="text-nexus-error">✗ {}</span>"#,
            e
        )),
    }
}

/// POST /api/actions/qbit/pause
async fn action_qbit_pause() -> HttpResponse {
    qbit_action("stop").await  // qBit v5+ uses "stop" instead of "pause"
}

/// POST /api/actions/qbit/resume
async fn action_qbit_resume() -> HttpResponse {
    qbit_action("start").await  // qBit v5+ uses "start" instead of "resume"
}

async fn qbit_action(action: &str) -> HttpResponse {
    let qbit_url = std::env::var("QBIT_URL").unwrap_or_else(|_| "http://localhost:8080".into());
    let qbit_user = std::env::var("QBIT_USERNAME").unwrap_or_else(|_| "admin".into());
    let qbit_pass = std::env::var("QBIT_PASSWORD").unwrap_or_else(|_| "".into());

    let client = match reqwest::Client::builder()
        .cookie_store(true)
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::Ok().body(format!(
                r#"<span class="text-nexus-error">✗ {}</span>"#, e
            ));
        }
    };

    // Login
    let login = client
        .post(format!("{}/api/v2/auth/login", qbit_url))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!("username={}&password={}", qbit_user, qbit_pass))
        .send()
        .await;

    if login.is_err() {
        return HttpResponse::Ok().body(
            r#"<span class="text-nexus-error">✗ qBit login failed</span>"#,
        );
    }

    // Perform action
    let endpoint = format!("{}/api/v2/torrents/{}", qbit_url, action);
    let result = client
        .post(&endpoint)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("hashes=all")
        .send()
        .await;

    match result {
        Ok(r) if r.status().is_success() => {
            let label = if action == "stop" { "paused" } else { "resumed" };
            HttpResponse::Ok().body(format!(
                r#"<span class="text-nexus-accent">✓ All downloads {}</span>"#,
                label
            ))
        }
        Ok(r) => HttpResponse::Ok().body(format!(
            r#"<span class="text-nexus-error">✗ HTTP {}</span>"#,
            r.status()
        )),
        Err(e) => HttpResponse::Ok().body(format!(
            r#"<span class="text-nexus-error">✗ {}</span>"#, e
        )),
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
            // Page
            .route("/", web::get().to(index))
            // Existing HTMX partials
            .route("/htmx/services", web::get().to(htmx_services))
            .route("/htmx/vitals", web::get().to(htmx_vitals))
            .route("/htmx/downloads", web::get().to(htmx_downloads))
            .route("/htmx/weather", web::get().to(htmx_weather))
            .route("/htmx/storage", web::get().to(htmx_storage))
            .route("/htmx/datetime", web::get().to(htmx_datetime))
            .route("/htmx/activity", web::get().to(htmx_activity))
            .route("/htmx/logs/{container}", web::get().to(htmx_logs))
            .route("/htmx/openclaw-logs", web::get().to(htmx_openclaw_logs))
            // New HTMX partials
            .route("/htmx/media-stats", web::get().to(htmx_media_stats))
            .route("/htmx/media-queue", web::get().to(htmx_media_queue))
            .route("/htmx/upcoming", web::get().to(htmx_upcoming))
            .route("/htmx/speedtest", web::get().to(htmx_speedtest))
            .route("/htmx/container-resources", web::get().to(htmx_container_resources))
            .route("/htmx/recently-added", web::get().to(htmx_recently_added))
            .route("/htmx/journal", web::get().to(htmx_journal))
            // Quick Actions
            .route("/api/actions/restart/{container}", web::post().to(action_restart_container))
            .route("/api/actions/qbit/pause", web::post().to(action_qbit_pause))
            .route("/api/actions/qbit/resume", web::post().to(action_qbit_resume))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
