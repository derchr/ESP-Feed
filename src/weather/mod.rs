// pub mod icons;

use anyhow::Result;
use embedded_svc::{
    http::client::{Client, Request},
    io::StdIO,
};
use esp_idf_svc::http::client::EspHttpClient;
use serde::Deserialize;
use std::io::BufReader;

const OPENWEATHER_API_KEY: &str = env!("OPENWEATHER_API_KEY");
const OPENWEATHER_LOCATION: &str = env!("OPENWEATHER_LOCATION");

#[derive(Deserialize, Debug)]
struct Weather {
    description: String,
    icon: String,
}

#[derive(Deserialize, Debug)]
struct Main {
    temp: f32,
    temp_min: f32,
    temp_max: f32,
    feels_like: f32,
    pressure: u32,
    humidity: u32,
}

#[derive(Deserialize, Debug)]
struct Sys {
    sunrise: u32,
    sunset: u32,
}

#[derive(Deserialize, Debug)]
pub struct WeatherPrimitive {
    weather: Vec<Weather>,
    main: Main,
    visibility: u32,
    name: String,
    sys: Sys,
}

pub struct WeatherReport<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub icon: &'a str,
    pub temp: f32,
    pub temp_min: f32,
    pub temp_max: f32,
    pub feels_like: f32,
    pub pressure: u32,
    pub humidity: u32,
    pub sunrise: u32,
    pub sunset: u32,
    pub visibility: u32,
}

pub struct WeatherController {
    current_report: Option<WeatherPrimitive>,
}

impl WeatherController {
    pub fn new() -> Self {
        Self {
            current_report: None,
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.current_report = Some(self.fetch_current_weather()?);
        Ok(())
    }

    pub fn current(&self) -> Option<WeatherReport> {
        let current_report = self.current_report.as_ref()?;
        let weather = current_report.weather.get(0)?;

        Some(WeatherReport {
            name: &current_report.name,
            description: &weather.description,
            icon: &weather.icon,
            temp: current_report.main.temp,
            temp_min: current_report.main.temp_min,
            temp_max: current_report.main.temp_max,
            feels_like: current_report.main.feels_like,
            pressure: current_report.main.pressure,
            humidity: current_report.main.humidity,
            sunrise: current_report.sys.sunrise,
            sunset: current_report.sys.sunset,
            visibility: current_report.visibility,
        })
    }

    fn fetch_current_weather(&self) -> Result<WeatherPrimitive> {
        let url = url::Url::parse(&format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&APPID={}&lang=de&units=metric",
            OPENWEATHER_LOCATION, OPENWEATHER_API_KEY
        ))
        .expect("Invalid Url");
        let mut client = EspHttpClient::new_default()?;
        let response = client.get(url)?.submit()?;
        let response_reader = BufReader::new(StdIO(&response));

        let report: WeatherPrimitive = serde_json::from_reader(response_reader)?;

        Ok(report)
    }
}
