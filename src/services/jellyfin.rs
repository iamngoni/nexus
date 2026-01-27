use serde::Serialize;
use reqwest::Client;

#[derive(Debug, Clone, Serialize)]
pub struct MediaStats {
    pub movie_count: u64,
    pub series_count: u64,
    pub episode_count: u64,
    pub song_count: u64,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecentItem {
    pub name: String,
    pub item_type: String,       // "Movie" or "Episode"
    pub series_name: String,     // empty for movies
    pub display_name: String,    // formatted for display
    pub date_added: String,      // relative time like "2h ago"
}

#[derive(Debug, Clone, Serialize)]
pub struct RecentlyAdded {
    pub items: Vec<RecentItem>,
    pub available: bool,
}

fn jellyfin_client() -> Result<(Client, String, String), Box<dyn std::error::Error>> {
    let url = std::env::var("JELLYFIN_URL").unwrap_or_else(|_| "http://localhost:8096".into());
    let key = std::env::var("JELLYFIN_API_KEY").unwrap_or_default();
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
    Ok((client, url, key))
}

fn auth_header(key: &str) -> String {
    format!("MediaBrowser Token=\"{}\"", key)
}

pub async fn get_media_stats() -> MediaStats {
    match fetch_media_stats().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Jellyfin media stats error: {}", e);
            MediaStats {
                movie_count: 0,
                series_count: 0,
                episode_count: 0,
                song_count: 0,
                available: false,
            }
        }
    }
}

async fn fetch_media_stats() -> Result<MediaStats, Box<dyn std::error::Error>> {
    let (client, url, key) = jellyfin_client()?;
    let resp: serde_json::Value = client
        .get(format!("{}/Items/Counts", url))
        .header("Authorization", auth_header(&key))
        .send()
        .await?
        .json()
        .await?;

    Ok(MediaStats {
        movie_count: resp["MovieCount"].as_u64().unwrap_or(0),
        series_count: resp["SeriesCount"].as_u64().unwrap_or(0),
        episode_count: resp["EpisodeCount"].as_u64().unwrap_or(0),
        song_count: resp["SongCount"].as_u64().unwrap_or(0),
        available: true,
    })
}

pub async fn get_recently_added() -> RecentlyAdded {
    match fetch_recently_added().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Jellyfin recently added error: {}", e);
            RecentlyAdded {
                items: vec![],
                available: false,
            }
        }
    }
}

fn relative_time(iso: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso) {
        let now = chrono::Utc::now();
        let diff = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
        let mins = diff.num_minutes();
        if mins < 1 {
            "just now".into()
        } else if mins < 60 {
            format!("{}m ago", mins)
        } else if mins < 1440 {
            format!("{}h ago", mins / 60)
        } else {
            format!("{}d ago", mins / 1440)
        }
    } else {
        String::new()
    }
}

async fn fetch_recently_added() -> Result<RecentlyAdded, Box<dyn std::error::Error>> {
    let (client, url, key) = jellyfin_client()?;
    let resp: serde_json::Value = client
        .get(format!(
            "{}/Items?SortBy=DateCreated&SortOrder=Descending&Limit=8&Recursive=true&IncludeItemTypes=Movie,Episode",
            url
        ))
        .header("Authorization", auth_header(&key))
        .send()
        .await?
        .json()
        .await?;

    let items = resp["Items"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|item| {
            let name = item["Name"].as_str().unwrap_or("Unknown").to_string();
            let item_type = item["Type"].as_str().unwrap_or("Unknown").to_string();
            let series_name = item["SeriesName"].as_str().unwrap_or("").to_string();
            let date_created = item["DateCreated"].as_str().unwrap_or("");

            let display_name = if item_type == "Episode" && !series_name.is_empty() {
                format!("{} — {}", series_name, name)
            } else {
                name.clone()
            };

            RecentItem {
                name,
                item_type,
                series_name,
                display_name,
                date_added: relative_time(date_created),
            }
        })
        .collect();

    Ok(RecentlyAdded {
        items,
        available: true,
    })
}
