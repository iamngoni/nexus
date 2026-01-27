use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct JournalEntry {
    pub timestamp: String,
    pub unit: String,
    pub message: String,
    pub level: String,  // "error", "warning", "info"
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemJournal {
    pub entries: Vec<JournalEntry>,
    pub has_entries: bool,
}

pub async fn get_journal() -> SystemJournal {
    // Try journalctl first (works on host or with mounted journal)
    if let Ok(journal) = fetch_journalctl().await {
        return journal;
    }

    // Fallback: read host syslog (mounted at /var/log/host-syslog)
    if let Ok(journal) = read_syslog().await {
        return journal;
    }

    SystemJournal {
        entries: vec![],
        has_entries: false,
    }
}

async fn fetch_journalctl() -> Result<SystemJournal, Box<dyn std::error::Error>> {
    let output = tokio::process::Command::new("journalctl")
        .args(["--no-pager", "-p", "err", "-n", "10", "--output=short-iso"])
        .output()
        .await?;

    if !output.status.success() {
        return Err("journalctl failed".into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries: Vec<JournalEntry> = stdout
        .lines()
        .filter(|line| !line.starts_with("--") && !line.is_empty())
        .map(|line| parse_journal_line(line))
        .collect();

    let has_entries = !entries.is_empty();
    Ok(SystemJournal {
        entries,
        has_entries,
    })
}

fn parse_journal_line(line: &str) -> JournalEntry {
    // Format: 2026-01-27T13:08:55+02:00 hostname unit[pid]: message
    let parts: Vec<&str> = line.splitn(4, ' ').collect();
    if parts.len() >= 4 {
        let timestamp = parts[0]
            .split('T')
            .nth(1)
            .and_then(|t| t.split('+').next().or_else(|| t.split('-').next()))
            .unwrap_or(parts[0])
            .to_string();
        let unit = parts[2]
            .split('[')
            .next()
            .unwrap_or(parts[2])
            .trim_end_matches(':')
            .to_string();
        let message = parts[3..]
            .join(" ")
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");

        JournalEntry {
            timestamp,
            unit,
            message,
            level: "error".into(),
        }
    } else {
        JournalEntry {
            timestamp: String::new(),
            unit: String::new(),
            message: line
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;"),
            level: "error".into(),
        }
    }
}

async fn read_syslog() -> Result<SystemJournal, Box<dyn std::error::Error>> {
    let content = tokio::fs::read_to_string("/var/log/host-syslog").await?;
    let lines: Vec<&str> = content.lines().collect();
    let start = if lines.len() > 200 { lines.len() - 200 } else { 0 };

    // Filter for error-like entries
    let error_keywords = ["error", "err", "fail", "crit", "alert", "emerg", "panic"];
    let entries: Vec<JournalEntry> = lines[start..]
        .iter()
        .filter(|line| {
            let lower = line.to_lowercase();
            error_keywords.iter().any(|kw| lower.contains(kw))
        })
        .rev()
        .take(10)
        .map(|line| {
            // Syslog format: Jan 27 13:08:55 hostname unit[pid]: message
            let parts: Vec<&str> = line.splitn(6, ' ').collect();
            let (timestamp, unit, message) = if parts.len() >= 6 {
                let ts = format!("{} {} {}", parts[0], parts[1], parts[2]);
                let u = parts[4]
                    .split('[')
                    .next()
                    .unwrap_or(parts[4])
                    .trim_end_matches(':');
                let msg = parts[5..].join(" ");
                (ts, u.to_string(), msg)
            } else {
                (String::new(), String::new(), line.to_string())
            };

            JournalEntry {
                timestamp,
                unit,
                message: message
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;"),
                level: "error".into(),
            }
        })
        .collect();

    let has_entries = !entries.is_empty();
    Ok(SystemJournal {
        entries,
        has_entries,
    })
}
