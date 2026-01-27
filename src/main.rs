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

/// HTMX partial: clawdbot live logs
async fn htmx_clawdbot_logs(data: web::Data<AppState>) -> HttpResponse {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let log_path = format!("/tmp/clawdbot/clawdbot-{}.log", today);

    let content = match tokio::fs::read_to_string(&log_path).await {
        Ok(text) => {
            // Take last 80 lines
            let lines: Vec<&str> = text.lines().collect();
            let start = if lines.len() > 80 { lines.len() - 80 } else { 0 };
            lines[start..].join("\n")
        }
        Err(_) => "No logs found for today.".to_string(),
    };

    let escaped = content
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    let mut ctx = tera::Context::new();
    ctx.insert("logs", &escaped);

    match data.tera.render("partials/clawdbot-logs.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(_) => {
            // Fallback inline render
            let html = format!(
                r#"<div class="flex items-center justify-between mb-3">
    <div class="flex items-center gap-2">
        <h2 class="text-sm font-semibold text-white">Son of Anton</h2>
        <span class="px-2 py-0.5 rounded-full text-[10px] font-mono" style="background: rgba(0,212,170,0.1); color: #00D4AA;">live</span>
    </div>
    <i data-lucide="terminal" class="w-4 h-4 text-nexus-muted"></i>
</div>
<pre id="clawdbot-log-pre" class="text-[11px] font-mono text-gray-400 bg-nexus-tile border border-nexus-tile-border rounded-tile p-3 overflow-auto max-h-[300px] whitespace-pre-wrap break-all leading-relaxed">{}</pre>
<script>
(function(){{ var el = document.getElementById('clawdbot-log-pre'); if(el) el.scrollTop = el.scrollHeight; }})();
</script>"#,
                escaped
            );
            HttpResponse::Ok().content_type("text/html").body(html)
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
