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
    let mut ctx = tera::Context::new();
    ctx.insert("services", &checker.get_statuses());
    ctx.insert("now", &chrono::Local::now().format("%H:%M").to_string());
    ctx.insert("date", &chrono::Local::now().format("%A, %B %e, %Y").to_string());

    match data.tera.render("index.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

/// HTMX partial: refreshes service status cards
async fn htmx_services(data: web::Data<AppState>) -> HttpResponse {
    let checker = data.checker.read().await;
    let mut ctx = tera::Context::new();
    ctx.insert("services", &checker.get_statuses());

    match data.tera.render("partials/services.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

/// HTMX partial: system vitals
async fn htmx_vitals(data: web::Data<AppState>) -> HttpResponse {
    let vitals = services::system::get_vitals().await;
    let mut ctx = tera::Context::new();
    ctx.insert("vitals", &vitals);

    match data.tera.render("partials/vitals.html", &ctx) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
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
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await
}
