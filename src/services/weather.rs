use serde::Serialize;
use reqwest::Client;

#[derive(Debug, Clone, Serialize)]
pub struct WeatherInfo {
    pub location: String,
    pub temp_c: String,
    pub condition: String,
    pub high_c: String,
    pub low_c: String,
    pub rain_chance: String,
    pub icon_name: String,  // lucide icon name
    pub humidity: String,
    pub wind_kph: String,
}

pub async fn get_weather() -> WeatherInfo {
    match fetch_weather().await {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Weather API error: {}", e);
            let location = std::env::var("WEATHER_LOCATION").unwrap_or_else(|_| "London".into());
            WeatherInfo {
                location,
                temp_c: "--".into(),
                condition: "Unavailable".into(),
                high_c: "--".into(),
                low_c: "--".into(),
                rain_chance: "0".into(),
                icon_name: "cloud-off".into(),
                humidity: "--".into(),
                wind_kph: "--".into(),
            }
        }
    }
}

fn weather_to_lucide_icon(code: i64) -> &'static str {
    // wttr.in weather codes → lucide icon names
    match code {
        113 => "sun",                    // Clear/Sunny
        116 => "cloud-sun",             // Partly Cloudy
        119 | 122 => "cloud",           // Cloudy / Overcast
        143 | 248 | 260 => "cloud-fog", // Mist / Fog / Freezing Fog
        176 | 263 | 266 | 293 | 296 => "cloud-drizzle", // Light rain/drizzle
        179 | 182 | 185 | 227 | 230 => "snowflake",     // Snow/Sleet
        200 | 386 | 389 | 392 | 395 => "cloud-lightning", // Thunder
        299 | 302 | 305 | 308 | 311 | 314 | 317 | 320 | 356 | 359 | 362 | 365 => "cloud-rain", // Heavy rain
        281 | 284 => "cloud-rain",      // Freezing drizzle
        323 | 326 | 329 | 332 | 335 | 338 | 350 | 368 | 371 | 374 | 377 => "snowflake", // Snow
        _ => "cloud-sun",
    }
}

async fn fetch_weather() -> Result<WeatherInfo, Box<dyn std::error::Error>> {
    let location = std::env::var("WEATHER_LOCATION").unwrap_or_else(|_| "London".into());

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let raw = client
        .get(format!("https://wttr.in/{}?format=j1", location))
        .header("User-Agent", "nexus-dashboard/1.0")
        .send()
        .await?;
    let resp: serde_json::Value = raw.json().await?;

    let current = &resp["current_condition"][0];
    let today = &resp["weather"][0];

    let temp = current["temp_C"].as_str().unwrap_or("--").to_string();
    let condition = current["weatherDesc"][0]["value"].as_str().unwrap_or("Unknown").to_string();
    let weather_code = current["weatherCode"].as_str().unwrap_or("116").parse::<i64>().unwrap_or(116);
    let humidity = current["humidity"].as_str().unwrap_or("--").to_string();
    let wind = current["windspeedKmph"].as_str().unwrap_or("--").to_string();
    
    let high = today["maxtempC"].as_str().unwrap_or("--").to_string();
    let low = today["mintempC"].as_str().unwrap_or("--").to_string();
    
    // Get rain chance from hourly data
    let rain_chance = today["hourly"]
        .as_array()
        .and_then(|hours| {
            hours.iter()
                .filter_map(|h| h["chanceofrain"].as_str()?.parse::<u32>().ok())
                .max()
        })
        .unwrap_or(0);

    Ok(WeatherInfo {
        location,
        temp_c: temp,
        condition,
        high_c: high,
        low_c: low,
        rain_chance: format!("{}", rain_chance),
        icon_name: weather_to_lucide_icon(weather_code).to_string(),
        humidity,
        wind_kph: wind,
    })
}
