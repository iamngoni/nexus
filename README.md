# Nexus — Homelab Dashboard

A sleek, dark-themed homelab dashboard built with Rust (Actix-web), HTMX, and Tailwind CSS. Live-refreshing widgets for service health, system vitals, downloads, weather, and more.

![Nexus Dashboard](https://img.shields.io/badge/Rust-1.88+-orange?logo=rust) ![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- **Service Monitoring** — Health checks for all your homelab services with response times and status indicators
- **System Vitals** — CPU, RAM, disk, and media drive usage with animated ring gauges
- **Downloads** — qBittorrent integration with active torrents, speeds, and progress bars
- **Weather** — Current conditions, high/low temps, rain chance, and wind speed
- **Container Logs** — View Docker container logs directly from the dashboard
- **Live Refresh** — All widgets auto-refresh via HTMX (10–60s intervals)
- **Dark Theme** — Custom color palette with glow effects and gradient accents

## Stack

- **Backend:** Rust + Actix-web
- **Frontend:** HTMX + Tailwind CSS (CDN) + Lucide Icons
- **Templating:** Tera
- **Fonts:** Inter + DM Mono

## Quick Start

### Docker (Recommended)

```bash
# Clone and configure
git clone https://github.com/iamngoni/nexus.git
cd nexus
cp .env.example .env
# Edit .env with your values

# Build and run
docker build -t nexus-dashboard:latest .
docker run -d \
  --name nexus \
  --network host \
  --env-file .env \
  -e TZ=Africa/Johannesburg \
  -v /mnt/media:/mnt/media:ro \
  -v /var/run/docker.sock:/var/run/docker.sock:ro \
  nexus-dashboard:latest
```

### Docker Compose

Add to your `docker-compose.yml`:

```yaml
nexus:
  image: nexus-dashboard:latest
  container_name: nexus
  env_file:
    - ./nexus/.env
  environment:
    - TZ=Africa/Johannesburg
    - RUST_LOG=info
  network_mode: host
  volumes:
    - /mnt/media:/mnt/media:ro
    - /var/run/docker.sock:/var/run/docker.sock:ro
  restart: unless-stopped
```

### Local Development

```bash
# Install Rust 1.88+
cp .env.example .env
# Edit .env with your values

cargo run
# → http://localhost:3000
```

## Configuration

All configuration is via environment variables. Copy `.env.example` to `.env` and adjust:

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST_IP` | IP address of your homelab host | `127.0.0.1` |
| `PORT` | Dashboard port | `3000` |
| `DASHBOARD_USER` | Name shown in the greeting | `Admin` |
| `QBIT_URL` | qBittorrent WebUI URL | `http://localhost:8080` |
| `QBIT_USERNAME` | qBittorrent username | `admin` |
| `QBIT_PASSWORD` | qBittorrent password | *(empty)* |
| `WEATHER_LOCATION` | Location for weather (wttr.in) | `London` |

## Volumes

| Path | Purpose |
|------|---------|
| `/mnt/media` | Media drive (read-only, for storage stats) |
| `/var/run/docker.sock` | Docker socket (read-only, for container logs) |

## Architecture

```
index.html (SSR initial render)
├── partials/datetime.html    → /htmx/datetime   (30s)
├── partials/services.html    → /htmx/services   (30s)
├── partials/vitals.html      → /htmx/vitals     (10s)
├── partials/downloads.html   → /htmx/downloads  (15s)
├── partials/weather.html     → /htmx/weather    (600s)
├── partials/activity.html    → /htmx/activity   (30s)
└── partials/storage.html     → /htmx/storage    (60s)

Log viewer: /htmx/logs/{container} → modal overlay
```

## Adding Services

Edit `src/models/config.rs` and add a new `ServiceDef` to the services vector. Each service needs:
- `name` — Display name
- `url` — Health check URL  
- `lucide_icon` — Icon name from [Lucide](https://lucide.dev/icons)
- `category` — Grouping label
- `container` — Docker container name (for logs)

## License

MIT
