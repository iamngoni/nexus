use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDef {
    pub name: String,
    pub url: String,
    pub health_url: String,
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
        let public_url = |key: &str, port: u16| {
            std::env::var(key).unwrap_or_else(|_| format!("http://{}:{}", ip, port))
        };
        let health_url = |key: &str, port: u16| {
            std::env::var(key).unwrap_or_else(|_| format!("http://{}:{}", ip, port))
        };

        AppConfig {
            services: vec![
                ServiceDef {
                    name: "Jellyfin".into(),
                    url: public_url("JELLYFIN_PUBLIC_URL", 8096),
                    health_url: health_url("JELLYFIN_URL", 8096),
                    icon: "🎬".into(),
                    lucide_icon: "tv".into(),
                    category: "Media".into(),
                    container: "jellyfin".into(),
                },
                ServiceDef {
                    name: "Sonarr".into(),
                    url: public_url("SONARR_PUBLIC_URL", 8989),
                    health_url: health_url("SONARR_URL", 8989),
                    icon: "📺".into(),
                    lucide_icon: "clapperboard".into(),
                    category: "Media".into(),
                    container: "sonarr".into(),
                },
                ServiceDef {
                    name: "Radarr".into(),
                    url: public_url("RADARR_PUBLIC_URL", 7878),
                    health_url: health_url("RADARR_URL", 7878),
                    icon: "🎥".into(),
                    lucide_icon: "film".into(),
                    category: "Media".into(),
                    container: "radarr".into(),
                },
                ServiceDef {
                    name: "Lidarr".into(),
                    url: public_url("LIDARR_PUBLIC_URL", 8686),
                    health_url: health_url("LIDARR_URL", 8686),
                    icon: "🎵".into(),
                    lucide_icon: "music".into(),
                    category: "Media".into(),
                    container: "lidarr".into(),
                },
                ServiceDef {
                    name: "Bazarr".into(),
                    url: public_url("BAZARR_PUBLIC_URL", 6767),
                    health_url: health_url("BAZARR_URL", 6767),
                    icon: "💬".into(),
                    lucide_icon: "languages".into(),
                    category: "Media".into(),
                    container: "bazarr".into(),
                },
                ServiceDef {
                    name: "Prowlarr".into(),
                    url: public_url("PROWLARR_PUBLIC_URL", 9696),
                    health_url: health_url("PROWLARR_URL", 9696),
                    icon: "🔍".into(),
                    lucide_icon: "search".into(),
                    category: "Media".into(),
                    container: "prowlarr".into(),
                },
                ServiceDef {
                    name: "Jellyseerr".into(),
                    url: public_url("JELLYSEERR_PUBLIC_URL", 5055),
                    health_url: health_url("JELLYSEERR_URL", 5055),
                    icon: "🎞️".into(),
                    lucide_icon: "ticket".into(),
                    category: "Media".into(),
                    container: "jellyseerr".into(),
                },
                ServiceDef {
                    name: "Kompressor".into(),
                    url: public_url("KOMPRESSOR_PUBLIC_URL", 8078),
                    health_url: health_url("KOMPRESSOR_URL", 8078),
                    icon: "🗜️".into(),
                    lucide_icon: "shrink".into(),
                    category: "Media".into(),
                    container: "kompressor".into(),
                },
                ServiceDef {
                    name: "qBittorrent".into(),
                    url: public_url("QBIT_PUBLIC_URL", 8080),
                    health_url: health_url("QBIT_URL", 8080),
                    icon: "📥".into(),
                    lucide_icon: "download".into(),
                    category: "Downloads".into(),
                    container: "qbittorrent".into(),
                },
                ServiceDef {
                    name: "MeTube".into(),
                    url: public_url("METUBE_PUBLIC_URL", 8081),
                    health_url: health_url("METUBE_URL", 8081),
                    icon: "📹".into(),
                    lucide_icon: "youtube".into(),
                    category: "Downloads".into(),
                    container: "metube".into(),
                },
                ServiceDef {
                    name: "JDownloader".into(),
                    url: public_url("JDOWNLOADER_PUBLIC_URL", 5800),
                    health_url: health_url("JDOWNLOADER_URL", 5800),
                    icon: "📦".into(),
                    lucide_icon: "download-cloud".into(),
                    category: "Downloads".into(),
                    container: "jdownloader".into(),
                },
                ServiceDef {
                    name: "Home Assistant".into(),
                    url: public_url("HOME_ASSISTANT_PUBLIC_URL", 8123),
                    health_url: health_url("HOME_ASSISTANT_URL", 8123),
                    icon: "🏠".into(),
                    lucide_icon: "house".into(),
                    category: "Automation".into(),
                    container: "homeassistant".into(),
                },
                ServiceDef {
                    name: "Uptime Kuma".into(),
                    url: public_url("UPTIME_KUMA_PUBLIC_URL", 3001),
                    health_url: health_url("UPTIME_KUMA_URL", 3001),
                    icon: "📡".into(),
                    lucide_icon: "activity".into(),
                    category: "Monitoring".into(),
                    container: "uptime-kuma".into(),
                },
                ServiceDef {
                    name: "Speedtest".into(),
                    url: public_url("SPEEDTEST_PUBLIC_URL", 8765),
                    health_url: health_url("SPEEDTEST_URL", 8765),
                    icon: "⚡".into(),
                    lucide_icon: "gauge".into(),
                    category: "Monitoring".into(),
                    container: "speedtest-tracker".into(),
                },
                ServiceDef {
                    name: "Traefik".into(),
                    url: public_url("TRAEFIK_PUBLIC_URL", 8280),
                    health_url: health_url("TRAEFIK_URL", 8280),
                    icon: "🔀".into(),
                    lucide_icon: "route".into(),
                    category: "Infrastructure".into(),
                    container: "traefik".into(),
                },
                ServiceDef {
                    name: "FlareSolverr".into(),
                    url: public_url("FLARESOLVERR_PUBLIC_URL", 8191),
                    health_url: health_url("FLARESOLVERR_URL", 8191),
                    icon: "🛡️".into(),
                    lucide_icon: "shield-off".into(),
                    category: "Infrastructure".into(),
                    container: "flaresolverr".into(),
                },
                ServiceDef {
                    name: "Nexus".into(),
                    url: public_url("NEXUS_PUBLIC_URL", 3000),
                    health_url: health_url("NEXUS_URL", 3000),
                    icon: "🖥️".into(),
                    lucide_icon: "layout-dashboard".into(),
                    category: "Infrastructure".into(),
                    container: "nexus".into(),
                },
            ],
        }
    }
}
