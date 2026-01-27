use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDef {
    pub name: String,
    pub url: String,
    pub icon: String,       // emoji (legacy)
    pub lucide_icon: String, // lucide icon name
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub services: Vec<ServiceDef>,
}

impl AppConfig {
    pub fn load() -> Self {
        AppConfig {
            services: vec![
                ServiceDef {
                    name: "Jellyfin".into(),
                    url: "http://100.78.244.68:8096".into(),
                    icon: "🎬".into(),
                    lucide_icon: "tv".into(),
                    category: "Media".into(),
                },
                ServiceDef {
                    name: "Sonarr".into(),
                    url: "http://100.78.244.68:8989".into(),
                    icon: "📺".into(),
                    lucide_icon: "clapperboard".into(),
                    category: "Media".into(),
                },
                ServiceDef {
                    name: "Radarr".into(),
                    url: "http://100.78.244.68:7878".into(),
                    icon: "🎥".into(),
                    lucide_icon: "film".into(),
                    category: "Media".into(),
                },
                ServiceDef {
                    name: "Bazarr".into(),
                    url: "http://100.78.244.68:6767".into(),
                    icon: "💬".into(),
                    lucide_icon: "languages".into(),
                    category: "Media".into(),
                },
                ServiceDef {
                    name: "Prowlarr".into(),
                    url: "http://100.78.244.68:9696".into(),
                    icon: "🔍".into(),
                    lucide_icon: "search".into(),
                    category: "Media".into(),
                },
                ServiceDef {
                    name: "Jellyseerr".into(),
                    url: "http://100.78.244.68:5055".into(),
                    icon: "🎞️".into(),
                    lucide_icon: "ticket".into(),
                    category: "Media".into(),
                },
                ServiceDef {
                    name: "qBittorrent".into(),
                    url: "http://100.78.244.68:8080".into(),
                    icon: "📥".into(),
                    lucide_icon: "download".into(),
                    category: "Downloads".into(),
                },
                ServiceDef {
                    name: "Home Assistant".into(),
                    url: "http://100.78.244.68:8123".into(),
                    icon: "🏠".into(),
                    lucide_icon: "house".into(),
                    category: "Automation".into(),
                },
                ServiceDef {
                    name: "Uptime Kuma".into(),
                    url: "http://100.78.244.68:3001".into(),
                    icon: "📡".into(),
                    lucide_icon: "activity".into(),
                    category: "Monitoring".into(),
                },
                ServiceDef {
                    name: "Speedtest".into(),
                    url: "http://100.78.244.68:8765".into(),
                    icon: "⚡".into(),
                    lucide_icon: "gauge".into(),
                    category: "Monitoring".into(),
                },
                ServiceDef {
                    name: "Traefik".into(),
                    url: "http://100.78.244.68:8880".into(),
                    icon: "🔀".into(),
                    lucide_icon: "route".into(),
                    category: "Infrastructure".into(),
                },
                ServiceDef {
                    name: "FlareSolverr".into(),
                    url: "http://100.78.244.68:8191".into(),
                    icon: "🛡️".into(),
                    lucide_icon: "shield-off".into(),
                    category: "Infrastructure".into(),
                },
            ],
        }
    }
}
