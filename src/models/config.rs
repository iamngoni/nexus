use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDef {
    pub name: String,
    pub url: String,
    pub icon: String,       // emoji (legacy)
    pub lucide_icon: String, // lucide icon name
    pub category: String,
    pub container: String,  // docker container name
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub services: Vec<ServiceDef>,
}

impl AppConfig {
    pub fn load() -> Self {
        let ip = std::env::var("HOST_IP").unwrap_or_else(|_| "127.0.0.1".into());
        AppConfig {
            services: vec![
                ServiceDef {
                    name: "Jellyfin".into(),
                    url: format!("http://{}:8096", ip),
                    icon: "🎬".into(),
                    lucide_icon: "tv".into(),
                    category: "Media".into(),
                    container: "jellyfin".into(),
                },
                ServiceDef {
                    name: "Sonarr".into(),
                    url: format!("http://{}:8989", ip),
                    icon: "📺".into(),
                    lucide_icon: "clapperboard".into(),
                    category: "Media".into(),
                    container: "sonarr".into(),
                },
                ServiceDef {
                    name: "Radarr".into(),
                    url: format!("http://{}:7878", ip),
                    icon: "🎥".into(),
                    lucide_icon: "film".into(),
                    category: "Media".into(),
                    container: "radarr".into(),
                },
                ServiceDef {
                    name: "Lidarr".into(),
                    url: format!("http://{}:8686", ip),
                    icon: "🎵".into(),
                    lucide_icon: "music".into(),
                    category: "Media".into(),
                    container: "lidarr".into(),
                },
                ServiceDef {
                    name: "Bazarr".into(),
                    url: format!("http://{}:6767", ip),
                    icon: "💬".into(),
                    lucide_icon: "languages".into(),
                    category: "Media".into(),
                    container: "bazarr".into(),
                },
                ServiceDef {
                    name: "Prowlarr".into(),
                    url: format!("http://{}:9696", ip),
                    icon: "🔍".into(),
                    lucide_icon: "search".into(),
                    category: "Media".into(),
                    container: "prowlarr".into(),
                },
                ServiceDef {
                    name: "Jellyseerr".into(),
                    url: format!("http://{}:5055", ip),
                    icon: "🎞️".into(),
                    lucide_icon: "ticket".into(),
                    category: "Media".into(),
                    container: "jellyseerr".into(),
                },
                ServiceDef {
                    name: "Kompressor".into(),
                    url: format!("http://{}:8078", ip),
                    icon: "🗜️".into(),
                    lucide_icon: "shrink".into(),
                    category: "Media".into(),
                    container: "kompressor".into(),
                },
                ServiceDef {
                    name: "qBittorrent".into(),
                    url: format!("http://{}:8080", ip),
                    icon: "📥".into(),
                    lucide_icon: "download".into(),
                    category: "Downloads".into(),
                    container: "qbittorrent".into(),
                },
                ServiceDef {
                    name: "MeTube".into(),
                    url: format!("http://{}:8081", ip),
                    icon: "📹".into(),
                    lucide_icon: "youtube".into(),
                    category: "Downloads".into(),
                    container: "metube".into(),
                },
                ServiceDef {
                    name: "JDownloader".into(),
                    url: format!("http://{}:5800", ip),
                    icon: "📦".into(),
                    lucide_icon: "download-cloud".into(),
                    category: "Downloads".into(),
                    container: "jdownloader".into(),
                },
                ServiceDef {
                    name: "Home Assistant".into(),
                    url: format!("http://{}:8123", ip),
                    icon: "🏠".into(),
                    lucide_icon: "house".into(),
                    category: "Automation".into(),
                    container: "homeassistant".into(),
                },
                ServiceDef {
                    name: "Uptime Kuma".into(),
                    url: format!("http://{}:3001", ip),
                    icon: "📡".into(),
                    lucide_icon: "activity".into(),
                    category: "Monitoring".into(),
                    container: "uptime-kuma".into(),
                },
                ServiceDef {
                    name: "Speedtest".into(),
                    url: format!("http://{}:8765", ip),
                    icon: "⚡".into(),
                    lucide_icon: "gauge".into(),
                    category: "Monitoring".into(),
                    container: "speedtest-tracker".into(),
                },
                ServiceDef {
                    name: "Traefik".into(),
                    url: format!("http://{}:8880", ip),
                    icon: "🔀".into(),
                    lucide_icon: "route".into(),
                    category: "Infrastructure".into(),
                    container: "traefik".into(),
                },
                ServiceDef {
                    name: "FlareSolverr".into(),
                    url: format!("http://{}:8191", ip),
                    icon: "🛡️".into(),
                    lucide_icon: "shield-off".into(),
                    category: "Infrastructure".into(),
                    container: "flaresolverr".into(),
                },
                ServiceDef {
                    name: "Nexus".into(),
                    url: format!("http://{}:3000", ip),
                    icon: "🖥️".into(),
                    lucide_icon: "layout-dashboard".into(),
                    category: "Infrastructure".into(),
                    container: "nexus".into(),
                },
            ],
        }
    }
}
