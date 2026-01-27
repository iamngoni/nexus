use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDef {
    pub name: String,
    pub url: String,
    pub icon: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub services: Vec<ServiceDef>,
}

impl AppConfig {
    pub fn load() -> Self {
        // TODO: Load from config file (nexus.toml)
        // For now, hardcode the homelab services
        AppConfig {
            services: vec![
                ServiceDef {
                    name: "Jellyfin".into(),
                    url: "http://100.78.244.68:8096".into(),
                    icon: "🎬".into(),
                    category: "Media".into(),
                },
                ServiceDef {
                    name: "Sonarr".into(),
                    url: "http://100.78.244.68:8989".into(),
                    icon: "📺".into(),
                    category: "Media".into(),
                },
                ServiceDef {
                    name: "Radarr".into(),
                    url: "http://100.78.244.68:7878".into(),
                    icon: "🎥".into(),
                    category: "Media".into(),
                },
                ServiceDef {
                    name: "Bazarr".into(),
                    url: "http://100.78.244.68:6767".into(),
                    icon: "💬".into(),
                    category: "Media".into(),
                },
                ServiceDef {
                    name: "Prowlarr".into(),
                    url: "http://100.78.244.68:9696".into(),
                    icon: "🔍".into(),
                    category: "Media".into(),
                },
                ServiceDef {
                    name: "Jellyseerr".into(),
                    url: "http://100.78.244.68:5055".into(),
                    icon: "🎞️".into(),
                    category: "Media".into(),
                },
                ServiceDef {
                    name: "qBittorrent".into(),
                    url: "http://100.78.244.68:8080".into(),
                    icon: "📥".into(),
                    category: "Downloads".into(),
                },
                ServiceDef {
                    name: "Home Assistant".into(),
                    url: "http://100.78.244.68:8123".into(),
                    icon: "🏠".into(),
                    category: "Automation".into(),
                },
                ServiceDef {
                    name: "Uptime Kuma".into(),
                    url: "http://100.78.244.68:3001".into(),
                    icon: "📡".into(),
                    category: "Monitoring".into(),
                },
                ServiceDef {
                    name: "Speedtest".into(),
                    url: "http://100.78.244.68:8765".into(),
                    icon: "⚡".into(),
                    category: "Monitoring".into(),
                },
                ServiceDef {
                    name: "FlareSolverr".into(),
                    url: "http://100.78.244.68:8191".into(),
                    icon: "🛡️".into(),
                    category: "Infrastructure".into(),
                },
            ],
        }
    }
}
