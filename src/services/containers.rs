use serde::Serialize;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct ContainerStat {
    pub name: String,
    pub cpu_percent: f64,
    pub mem_usage: String,
    pub mem_percent: f64,
    pub net_io: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContainerResources {
    pub containers: Vec<ContainerStat>,
}

pub async fn get_container_resources() -> ContainerResources {
    match fetch_container_stats().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Docker stats error: {}", e);
            ContainerResources {
                containers: vec![],
            }
        }
    }
}

async fn fetch_container_stats() -> Result<ContainerResources, Box<dyn std::error::Error>> {
    let output = Command::new("docker")
        .args(["stats", "--no-stream", "--format", "{{json .}}"])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut containers: Vec<ContainerStat> = stdout
        .lines()
        .filter_map(|line| {
            let v: serde_json::Value = serde_json::from_str(line).ok()?;
            let name = v["Name"].as_str().unwrap_or("").to_string();
            let cpu_str = v["CPUPerc"].as_str().unwrap_or("0%");
            let cpu_percent = cpu_str.trim_end_matches('%').parse::<f64>().unwrap_or(0.0);
            let mem_usage = v["MemUsage"].as_str().unwrap_or("0B / 0B").to_string();
            let mem_perc_str = v["MemPerc"].as_str().unwrap_or("0%");
            let mem_percent = mem_perc_str.trim_end_matches('%').parse::<f64>().unwrap_or(0.0);
            let net_io = v["NetIO"].as_str().unwrap_or("0B / 0B").to_string();

            Some(ContainerStat {
                name,
                cpu_percent,
                mem_usage,
                mem_percent,
                net_io,
            })
        })
        .collect();

    // Sort by CPU descending, then memory
    containers.sort_by(|a, b| {
        b.cpu_percent
            .partial_cmp(&a.cpu_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                b.mem_percent
                    .partial_cmp(&a.mem_percent)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });

    // Limit to top 20
    containers.truncate(20);

    Ok(ContainerResources { containers })
}
