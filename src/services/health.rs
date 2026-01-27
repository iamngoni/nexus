use serde::Serialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::models::config::ServiceDef;

#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
    pub name: String,
    pub url: String,
    pub icon: String,
    pub category: String,
    pub status: String,       // "up", "down", "unknown"
    pub response_ms: u64,
    pub last_checked: String,
}

pub struct ServiceChecker {
    statuses: HashMap<String, ServiceStatus>,
}

impl ServiceChecker {
    pub fn new(services: &[ServiceDef]) -> Self {
        let mut statuses = HashMap::new();
        for s in services {
            statuses.insert(s.name.clone(), ServiceStatus {
                name: s.name.clone(),
                url: s.url.clone(),
                icon: s.icon.clone(),
                category: s.category.clone(),
                status: "unknown".into(),
                response_ms: 0,
                last_checked: "never".into(),
            });
        }
        ServiceChecker { statuses }
    }

    pub async fn check_all(&mut self, services: &[ServiceDef]) {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        for s in services {
            let start = Instant::now();
            let result = client.get(&s.url).send().await;
            let elapsed = start.elapsed().as_millis() as u64;

            let status = match result {
                Ok(resp) if resp.status().is_success() || resp.status().is_redirection() => "up",
                Ok(resp) if resp.status().as_u16() == 401 || resp.status().as_u16() == 403 => "up", // auth-protected but alive
                Ok(_) => "degraded",
                Err(_) => "down",
            };

            if let Some(entry) = self.statuses.get_mut(&s.name) {
                entry.status = status.into();
                entry.response_ms = elapsed;
                entry.last_checked = chrono::Local::now().format("%H:%M:%S").to_string();
            }
        }
    }

    pub fn get_statuses(&self) -> Vec<&ServiceStatus> {
        let mut list: Vec<&ServiceStatus> = self.statuses.values().collect();
        list.sort_by(|a, b| a.category.cmp(&b.category).then(a.name.cmp(&b.name)));
        list
    }
}
