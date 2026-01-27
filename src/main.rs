mod services;
mod models;

use actix_web::{web, App, HttpServer, HttpResponse};
use actix_files as fs;
use tera::Tera;
use std::sync::Arc;
use tokio::sync::RwLock;

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

    let mut ctx = tera::Context::new();
    ctx.insert("services", &statuses);
    ctx.insert("online_count", &online_count);
    ctx.insert("total_count", &statuses.len());

    // Greeting based on time of day
    let hour = chrono::Local::now().format("%H").to_string().parse::<u32>().unwrap_or(12);
    let greeting = match hour {
        5..=11 => "Good morning",
        12..=17 => "Good afternoon",
        18..=22 => "Good evening",
        _ => "Good night",
    };
    ctx.insert("greeting", greeting);
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
    ctx.insert("greeting", greeting);
    ctx.insert("time", &chrono::Local::now().format("%H:%M").to_string());
    ctx.insert("date", &chrono::Local::now().format("%a %d %b").to_string());

    match data.tera.render("partials/datetime.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Partial error: {}", e)),
    }
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
    println!("🚀 Nexus starting on http://0.0.0.0:3000");

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
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await
}
