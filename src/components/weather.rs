use anyhow::{Context, Result};
use crate::component::Component;
use crate::config::WeatherConfig;
use crate::output::{ComponentOutput, OutputRow};
use crate::styled_text::StyledText;
use reqwest::blocking::Client;
use std::collections::HashMap;

const GEOLOCATE_URL: &str = "http://ip-api.com/json?fields=status,message,lat,lon"

pub struct WeatherCopmonent<'a> {
    config: &'a WeatherConfig,
    colour: &'a str,
    show_section_titles: &'a bool,
}

impl<'a> WeatherComponent<'a> {
    pub fn new(
        config: &'a WeatherConfig,
        colour: &'a str,
        show_section_titles: &'a bool,
    ) -> Self {
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
                ouput.rows.push(
                    OutputRow::Section(
                        StyledText::new(self.title())
                        .bold().underline().apply_style(self.colour)
                    )
                )
            } else {
                output.rows.push(OutputRow::Section(
                    StyledText::new(self.title())
                    .bold()
                    .underline()
                ))
            }
        }

        let weather_codes = HashMap::from([
            ("0", ("Clear sky", "☀️")),
            ("1", ("Mainly clear", "🌤️")),
            ("2", ("Partly cloudy", "⛅")),
            ("3", ("Overcast", "☁️")),
            ("45", ("Fog", "🌫️")),
            ("48", ("Depositing rime fog", "🌫️")),
            ("51", ("Light drizzle", "🌦️")),
            ("53", ("Moderate drizzle", "🌦️")),
            ("55", ("Dense drizzle", "🌧️")),
            ("56", ("Light freezing drizzle", "🌧️")),
            ("57", ("Dense freezing drizzle", "🌧️")),
            ("61", ("Slight rain", "🌦️")),
            ("63", ("Moderate rain", "🌧️")),
            ("65", ("Heavy rain", "🌧️")),
            ("66", ("Light freezing rain", "🌧️")),
            ("67", ("Heavy freezing rain", "🌧️")),
            ("71", ("Slight snowfall", "🌨️")),
            ("73", ("Moderate snowfall", "🌨️")),
            ("75", ("Heavy snowfall", "❄️")),
            ("77", ("Snow grains", "🌨️")),
            ("80", ("Slight rain showers", "🌦️")),
            ("81", ("Moderate rain showers", "🌧️")),
            ("82", ("Violent rain showers", "⛈️")),
            ("85", ("Slight snow showers", "🌨️")),
            ("86", ("Heavy snow showers", "❄️")),
            ("95", ("Thunderstorm", "⛈️")),
            ("96", ("Thunderstorm with slight hail", "⛈️")),
            ("97", ("Heavy thunderstorm", "🌩️")),
            ("99", ("Thunderstorm with heavy hail", "⛈️")),
        ]);

        let client = Client::builder().user_agent("fetchall").build()?;
        let location: (u64, u64) = client.get(GEOLOCATE_URL).send()?.error_for_status().context("Failed to make request to ip-api. You may have hit rate-limits, please wait a bit and try again.")?.json::<serde_json::Value>()?.map(|s| (s.get("lat").and_then(|j| j as u64).unwrap_or(0), s.get("long").and_then(|a| a as u64).unwrap_or(0)));
        let weather = client.get(format!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max,precipitation_sum,precipitation_hours,rain_sum,showers_sum&current=temperature_2m,apparent_temperature,precipitation,rain,showers,weather_code&timezone=auto", location.0, location.1)).send().error_for_status().context("Failed to get weather data")?.json::<serde_json::Value>().context("Failed to convert to weather data")?;
        let cur_temp = weather.get("current")?.get("temperature_2m")?;
        let cur_condition = weather_codes.get(weather.get("current")?.get("weather_code")?)?;
        let cur_precipitation =


    }
}
