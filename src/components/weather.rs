use crate::component::Component;
use crate::config::WeatherConfig;
use crate::helpers::deg_to_cardinal;
use crate::output::{ComponentOutput, OutputRow};
use crate::styled_text::StyledText;
use anyhow::{Context, Result};
use chrono::{Duration, Local, NaiveDateTime};
use reqwest::blocking::Client;
use std::collections::HashMap;

const GEOLOCATE_URL: &str = "http://ip-api.com/json?fields=status,message,lat,lon";

pub struct WeatherComponent<'a> {
    config: &'a WeatherConfig,
    colour: &'a str,
    show_section_titles: &'a bool,
}

impl<'a> WeatherComponent<'a> {
    pub fn new(config: &'a WeatherConfig, colour: &'a str, show_section_titles: &'a bool) -> Self {
        Self {
            config,
            colour,
            show_section_titles,
        }
    }
}

impl Component for WeatherComponent<'_> {
    fn title(&self) -> &str {
        "Weather"
    }
    fn collect(&self) -> Result<ComponentOutput> {
        let mut output = ComponentOutput { rows: Vec::new() };
        if !self.config.enabled {
            return Ok(output);
        }
        if *self.show_section_titles {
            if self.colour != "auto" {
                output.rows.push(OutputRow::Section(
                    StyledText::new(self.title())
                        .bold()
                        .underline()
                        .apply_style(self.colour),
                ))
            } else {
                output.rows.push(OutputRow::Section(
                    StyledText::new(self.title()).bold().underline(),
                ))
            }
        }

        let weather_codes: HashMap<u8, (&str, &str)> = HashMap::from([
            (0, ("Clear sky", "☀️")),
            (1, ("Mainly clear", "🌤️")),
            (2, ("Partly cloudy", "⛅")),
            (3, ("Overcast", "☁️")),
            (45, ("Fog", "🌫️")),
            (48, ("Depositing rime fog", "🌫️")),
            (51, ("Light drizzle", "🌦️")),
            (53, ("Moderate drizzle", "🌦️")),
            (55, ("Dense drizzle", "🌧️")),
            (56, ("Light freezing drizzle", "🌧️")),
            (57, ("Dense freezing drizzle", "🌧️")),
            (61, ("Slight rain", "🌦️")),
            (63, ("Moderate rain", "🌧️")),
            (65, ("Heavy rain", "🌧️")),
            (66, ("Light freezing rain", "🌧️")),
            (67, ("Heavy freezing rain", "🌧️")),
            (71, ("Slight snowfall", "🌨️")),
            (73, ("Moderate snowfall", "🌨️")),
            (75, ("Heavy snowfall", "❄️")),
            (77, ("Snow grains", "🌨️")),
            (80, ("Slight rain showers", "🌦️")),
            (81, ("Moderate rain showers", "🌧️")),
            (82, ("Violent rain showers", "⛈️")),
            (85, ("Slight snow showers", "🌨️")),
            (86, ("Heavy snow showers", "❄️")),
            (95, ("Thunderstorm", "⛈️")),
            (96, ("Thunderstorm with slight hail", "⛈️")),
            (97, ("Heavy thunderstorm", "🌩️")),
            (99, ("Thunderstorm with heavy hail", "⛈️")),
        ]);

        let client = Client::builder().user_agent("fetchall").build()?;

        let location = client.get(GEOLOCATE_URL).send()?.error_for_status().context("Failed to make request to ip-api. You may have hit rate-limits, please wait a bit and try again.")?.json::<serde_json::Value>()?;
        let coords = (
            location.get("lat").and_then(|j| j.as_f64()).unwrap_or(0.0),
            location.get("lon").and_then(|a| a.as_f64()).unwrap_or(0.0),
        );

        let weather = client.get(
            format!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&daily=precipitation_sum,sunset,sunrise,weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max,precipitation_sum,precipitation_hours,rain_sum,showers_sum&current=wind_speed_10m,relative_humidity_2m,wind_direction_10m,wind_gusts_10m,temperature_2m,apparent_temperature,precipitation,rain,showers,weather_code&hourly=uv_index&timezone=auto&forecast_days=1", coords.0, coords.1)
        )
        .send()?
        .error_for_status()
        .context("Failed to get weather data")?
        .json::<serde_json::Value>()
        .context("Failed to convert to weather data")?;

        if self.config.show_current {
            let cur_temp = weather
                .get("current")
                .and_then(|c| c.get("temperature_2m"))
                .and_then(|t| t.as_f64())
                .unwrap_or(0.0);

            let cur_condition = weather_codes
                .get(
                    &(weather
                        .get("current")
                        .and_then(|c| c.get("weather_code"))
                        .and_then(|w| w.as_u64())
                        .unwrap_or(0) as u8),
                )
                .unwrap_or(&("Unknown", "❓"));
            let cur_feel = weather
                .get("current")
                .and_then(|c| c.get("apparent_temperature"))
                .and_then(|x| x.as_f64())
                .unwrap_or(0.0);
            let cur_humidity = weather
                .get("current")
                .and_then(|c| c.get("relative_humidity_2m"))
                .and_then(|x| x.as_f64())
                .unwrap_or(0.0);

            if self.colour != "auto" {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("current").bold().apply_style(self.colour),
                    value: StyledText::new(format!(
                        "{} {:.1}°C | feels {:.1}° | {:.0}% RH",
                        cur_condition.1, cur_temp, cur_feel, cur_humidity
                    ))
                    .apply_style(self.colour),
                })
            } else {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("current").bold().red(),
                    value: StyledText::new(format!(
                        "{} {:.1}°C | feels {:.1}° | {:.0}% RH",
                        cur_condition.1, cur_temp, cur_feel, cur_humidity
                    )),
                })
            }
        }

        if self.config.show_today {
            let today_min = weather
                .get("daily")
                .and_then(|c| c.get("temperature_2m_min"))
                .and_then(|c| c.as_array())
                .and_then(|d| d.iter().next())
                .and_then(|f| f.as_f64())
                .unwrap_or(0.0);
            let today_max = weather
                .get("daily")
                .and_then(|c| c.get("temperature_2m_max"))
                .and_then(|c| c.as_array())
                .and_then(|d| d.iter().next())
                .and_then(|f| f.as_f64())
                .unwrap_or(0.0);
            let today_prob_rain = weather
                .get("daily")
                .and_then(|c| c.get("precipitation_probability_max"))
                .and_then(|c| c.as_array())
                .and_then(|d| d.iter().next())
                .and_then(|f| f.as_u64())
                .unwrap_or(0);
            let today_rain_sum = weather
                .get("daily")
                .and_then(|c| c.get("precipitation_sum"))
                .and_then(|c| c.as_array())
                .and_then(|d| d.iter().next())
                .and_then(|f| f.as_f64())
                .unwrap_or(0.0);
            if self.colour != "auto" {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("today").bold().apply_style(self.colour),
                    value: StyledText::new(format!(
                        "{:.1}°/{:.1}° | rain {:.0}%/{:.1}mm",
                        today_min, today_max, today_prob_rain, today_rain_sum
                    )),
                })
            } else {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("today").bold().bright_blue(),
                    value: StyledText::new(format!(
                        "{:.1}°/{:.1}° | rain {:.0}%/{:.1}mm",
                        today_min, today_max, today_prob_rain, today_rain_sum
                    )),
                })
            }
        }

        if self.config.show_wind {
            let cur_wind = weather
                .get("current")
                .and_then(|c| c.get("wind_speed_10m"))
                .and_then(|x| x.as_f64())
                .unwrap_or(0.0);
            let cur_wind_dir = weather
                .get("current")
                .and_then(|c| c.get("wind_direction_10m"))
                .and_then(|x| x.as_f64())
                .unwrap_or(0.0);
            let cur_wind_gust = weather
                .get("current")
                .and_then(|c| c.get("wind_gusts_10m"))
                .and_then(|x| x.as_f64())
                .unwrap_or(0.0);
            if self.colour != "auto" {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("wind").bold().apply_style(self.colour),
                    value: StyledText::new(format!(
                        "{:.1} km/h {} | gusts {:.1} km/h°",
                        cur_wind,
                        deg_to_cardinal(cur_wind_dir),
                        cur_wind_gust
                    ))
                    .apply_style(self.colour),
                })
            } else {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("wind").bold().bright_cyan(),
                    value: StyledText::new(format!(
                        "{:.1} km/h {} | gusts {:.1} km/h",
                        cur_wind,
                        deg_to_cardinal(cur_wind_dir),
                        cur_wind_gust
                    )),
                })
            }
        }

        if self.config.show_sun {
            let now = Local::now().format("%Y-%m-%dT%H:00").to_string();

            let uv_index = weather
                .get("hourly")
                .and_then(|c| c.get("time"))
                .and_then(|c| c.as_array())
                .and_then(|c| c.iter().position(|k| k == &now))
                .map(|i| {
                    weather
                        .get("hourly")
                        .and_then(|c| c.get("uv_index"))
                        .and_then(|f| f.as_array())
                        .map(|d| d[i].as_f64().unwrap_or(0.0))
                        .unwrap_or(0.0)
                })
                .unwrap_or(0.0);
            let sunset = NaiveDateTime::parse_from_str(
                weather
                    .get("daily")
                    .and_then(|c| c.get("sunset"))
                    .and_then(|c| c.as_array())
                    .and_then(|d| d.iter().next())
                    .and_then(|f| f.as_str())
                    .unwrap_or("1970-01-01T00:00"),
                "%Y-%m-%dT%H:%M",
            )?;
            let sunrise = NaiveDateTime::parse_from_str(
                weather
                    .get("daily")
                    .and_then(|c| c.get("sunrise"))
                    .and_then(|c| c.as_array())
                    .and_then(|d| d.iter().next())
                    .and_then(|f| f.as_str())
                    .unwrap_or("1970-01-01T00:00"),
                "%Y-%m-%dT%H:%M",
            )?;
            if self.colour != "auto" {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("sun").bold().apply_style(self.colour),
                    value: StyledText::new(format!(
                        "{:.1} UV | {}",
                        uv_index,
                        format_sunset_time(sunset, sunrise)
                    ))
                    .apply_style(self.colour),
                })
            } else {
                output.rows.push(OutputRow::Info {
                    label: StyledText::new("sun").bold().yellow(),
                    value: StyledText::new(format!(
                        "{:.1} UV | {}",
                        uv_index,
                        format_sunset_time(sunset, sunrise)
                    )),
                })
            }
        }

        Ok(output)
    }
}

fn format_sunset_time(sunset: NaiveDateTime, sunrise: NaiveDateTime) -> String {
    let now = Local::now().naive_local();
    if now >= sunset {
        let mins = (sunrise + Duration::days(1) - now).num_minutes();
        if mins <= 1 {
            "🌅 sunrise now".to_string()
        } else if mins < 55 {
            format!("🌇 sunrise in {} mins", mins)
        } else if mins <= 65 {
            "🌆sunrise in 1 hr".to_string()
        } else {
            format!(
                "🌃 sunrise at {}",
                (sunrise + Duration::days(1)).format("%I:%M %p")
            )
        }
    } else if now < sunrise {
        let mins = (sunrise - now).num_minutes();
        if mins <= 1 {
            "🌅 sunrise now".to_string()
        } else if mins < 55 {
            format!("🌇 sunrise in {} mins", mins)
        } else if mins <= 65 {
            "🌆 sunrise in 1 hr".to_string()
        } else {
            format!("🌃 sunrise at {}", sunrise.format("%I:%M %p"))
        }
    } else {
        let mins = (sunset - now).num_minutes();
        if mins <= 1 {
            "🌆 sunset now".to_string()
        } else if mins < 55 {
            format!("🌇 sunset in {} mins", mins)
        } else if mins <= 65 {
            "🌄 sunset in 1 hr".to_string()
        } else {
            format!("☀️ sunset at {}", sunset.format("%I:%M %p"))
        }
    }
}
