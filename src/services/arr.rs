use serde::Serialize;
use reqwest::Client;

// --- Queue ---

#[derive(Debug, Clone, Serialize)]
pub struct QueueItem {
    pub title: String,
    pub status: String,          // "downloading", "importing", "queued", etc.
    pub progress: f64,           // 0.0 - 100.0
    pub source: String,          // "sonarr" or "radarr"
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaQueue {
    pub items: Vec<QueueItem>,
    pub total_count: usize,
}

pub async fn get_media_queue() -> MediaQueue {
    let (sonarr_items, sonarr_total) = fetch_queue("sonarr").await;
    let (radarr_items, radarr_total) = fetch_queue("radarr").await;

    let mut items = sonarr_items;
    items.extend(radarr_items);

    // Sort: downloading first, then by progress ascending
    items.sort_by(|a, b| {
        let a_dl = a.status == "downloading";
        let b_dl = b.status == "downloading";
        b_dl.cmp(&a_dl)
            .then(a.progress.partial_cmp(&b.progress).unwrap_or(std::cmp::Ordering::Equal))
    });

    items.truncate(10);

    MediaQueue {
        total_count: sonarr_total + radarr_total,
        items,
    }
}

async fn fetch_queue(source: &str) -> (Vec<QueueItem>, usize) {
    let (url, key) = match source {
        "sonarr" => (
            std::env::var("SONARR_URL").unwrap_or_else(|_| "http://localhost:8989".into()),
            std::env::var("SONARR_API_KEY").unwrap_or_default(),
        ),
        "radarr" => (
            std::env::var("RADARR_URL").unwrap_or_else(|_| "http://localhost:7878".into()),
            std::env::var("RADARR_API_KEY").unwrap_or_default(),
        ),
        _ => return (vec![], 0),
    };

    let client = match Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => return (vec![], 0),
    };

    let resp = match client
        .get(format!("{}/api/v3/queue?page=1&pageSize=10", url))
        .header("X-Api-Key", &key)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} queue error: {}", source, e);
            return (vec![], 0);
        }
    };

    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(_) => return (vec![], 0),
    };

    let total = json["totalRecords"].as_u64().unwrap_or(0) as usize;
    let records = json["records"].as_array().cloned().unwrap_or_default();

    let items: Vec<QueueItem> = records
        .iter()
        .map(|r| {
            let title = r["title"].as_str().unwrap_or("Unknown").to_string();
            let state = r["trackedDownloadState"]
                .as_str()
                .unwrap_or(r["status"].as_str().unwrap_or("queued"))
                .to_string();

            let size = r["size"].as_f64().unwrap_or(1.0);
            let sizeleft = r["sizeleft"].as_f64().unwrap_or(0.0);
            let progress = if size > 0.0 {
                ((size - sizeleft) / size * 100.0).min(100.0)
            } else {
                0.0
            };

            // Clean up the title for display
            let short_title = if title.len() > 60 {
                format!("{}…", &title[..57])
            } else {
                title
            };

            QueueItem {
                title: short_title,
                status: state,
                progress,
                source: source.to_string(),
            }
        })
        .collect();

    (items, total)
}

// --- Calendar / Upcoming ---

#[derive(Debug, Clone, Serialize)]
pub struct CalendarItem {
    pub title: String,           // Episode title or movie title
    pub series_title: String,    // Series name (empty for movies)
    pub display_title: String,   // Formatted for display
    pub air_date: String,        // Formatted date
    pub episode_info: String,    // "S01E04" or empty for movies
    pub network: String,         // Network/studio
    pub source: String,          // "sonarr" or "radarr"
}

#[derive(Debug, Clone, Serialize)]
pub struct UpcomingReleases {
    pub items: Vec<CalendarItem>,
}

pub async fn get_upcoming() -> UpcomingReleases {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let end = (chrono::Local::now() + chrono::Duration::days(7))
        .format("%Y-%m-%d")
        .to_string();

    let sonarr = fetch_sonarr_calendar(&today, &end).await;
    let radarr = fetch_radarr_calendar(&today, &end).await;

    let mut items = sonarr;
    items.extend(radarr);

    // Sort by air date
    items.sort_by(|a, b| a.air_date.cmp(&b.air_date));
    items.truncate(10);

    UpcomingReleases { items }
}

async fn fetch_sonarr_calendar(start: &str, end: &str) -> Vec<CalendarItem> {
    let url = std::env::var("SONARR_URL").unwrap_or_else(|_| "http://localhost:8989".into());
    let key = std::env::var("SONARR_API_KEY").unwrap_or_default();

    let client = match Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let resp = match client
        .get(format!(
            "{}/api/v3/calendar?start={}&end={}&includeSeries=true",
            url, start, end
        ))
        .header("X-Api-Key", &key)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Sonarr calendar error: {}", e);
            return vec![];
        }
    };

    let json: Vec<serde_json::Value> = match resp.json().await {
        Ok(j) => j,
        Err(_) => return vec![],
    };

    json.iter()
        .map(|ep| {
            let title = ep["title"].as_str().unwrap_or("Unknown").to_string();
            let series = &ep["series"];
            let series_title = series["title"].as_str().unwrap_or("").to_string();
            let network = series["network"].as_str().unwrap_or("").to_string();
            let season = ep["seasonNumber"].as_u64().unwrap_or(0);
            let episode = ep["episodeNumber"].as_u64().unwrap_or(0);
            let episode_info = format!("S{:02}E{:02}", season, episode);

            let air_date_raw = ep["airDateUtc"].as_str().unwrap_or("");
            let air_date = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(air_date_raw) {
                dt.with_timezone(&chrono::Local)
                    .format("%a %d %b")
                    .to_string()
            } else {
                ep["airDate"].as_str().unwrap_or("TBA").to_string()
            };

            let display_title = if !series_title.is_empty() {
                format!("{} — {}", series_title, title)
            } else {
                title.clone()
            };

            CalendarItem {
                title,
                series_title,
                display_title,
                air_date,
                episode_info,
                network,
                source: "sonarr".into(),
            }
        })
        .collect()
}

async fn fetch_radarr_calendar(start: &str, end: &str) -> Vec<CalendarItem> {
    let url = std::env::var("RADARR_URL").unwrap_or_else(|_| "http://localhost:7878".into());
    let key = std::env::var("RADARR_API_KEY").unwrap_or_default();

    let client = match Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let resp = match client
        .get(format!(
            "{}/api/v3/calendar?start={}&end={}",
            url, start, end
        ))
        .header("X-Api-Key", &key)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Radarr calendar error: {}", e);
            return vec![];
        }
    };

    let json: Vec<serde_json::Value> = match resp.json().await {
        Ok(j) => j,
        Err(_) => return vec![],
    };

    json.iter()
        .map(|movie| {
            let title = movie["title"].as_str().unwrap_or("Unknown").to_string();
            let studio = movie["studio"].as_str().unwrap_or("").to_string();

            // Try digitalRelease, then physicalRelease, then inCinemas
            let air_date_raw = movie["digitalRelease"]
                .as_str()
                .or_else(|| movie["physicalRelease"].as_str())
                .or_else(|| movie["inCinemas"].as_str())
                .unwrap_or("");
            let air_date = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(air_date_raw) {
                dt.with_timezone(&chrono::Local)
                    .format("%a %d %b")
                    .to_string()
            } else {
                "TBA".to_string()
            };

            CalendarItem {
                title: title.clone(),
                series_title: String::new(),
                display_title: title,
                air_date,
                episode_info: String::new(),
                network: studio,
                source: "radarr".into(),
            }
        })
        .collect()
}
