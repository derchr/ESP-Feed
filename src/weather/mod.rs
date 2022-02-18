mod openweather_types;

use anyhow::Result;
use embedded_svc::{
    http::client::{Client, Request},
    io::StdIO,
};
use esp_idf_svc::http::client::EspHttpClient;
use openweather_types::{Coord, Daily, Hourly, OpenWeather, OpenWeatherOnecall};
use std::{
    convert::{TryFrom, TryInto},
    io::BufReader,
};

const OPENWEATHER_API_KEY: &str = env!("OPENWEATHER_API_KEY");

#[derive(Default)]
pub struct WeatherReport<'a> {
    pub name: Option<&'a str>,
    pub description: &'a str,
    pub icon: &'a str,
    pub temp: f32,
    pub temp_min: Option<f32>,
    pub temp_max: Option<f32>,
    pub feels_like: Option<f32>,
    pub pressure: f32,
    pub humidity: f32,
    pub sunrise: Option<i64>,
    pub sunset: Option<i64>,
    pub visibility: Option<f32>,
    pub dt: i64,
}

impl<'a> TryFrom<&'a OpenWeather> for WeatherReport<'a> {
    type Error = anyhow::Error;

    fn try_from(item: &'a OpenWeather) -> Result<Self> {
        let weather_description = item.weather.get(0).unwrap(); // TODO
        let main = &item.main;

        Ok(Self {
            name: Some(&item.name),
            description: &weather_description.description,
            icon: &weather_description.icon,
            temp: main.temp,
            temp_min: Some(main.temp_min),
            temp_max: Some(main.temp_max),
            feels_like: Some(main.feels_like),
            pressure: main.pressure,
            humidity: main.humidity,
            sunrise: Some(item.sys.sunrise),
            sunset: Some(item.sys.sunset),
            visibility: Some(item.visibility),
            dt: item.dt,
        })
    }
}

impl<'a> TryFrom<&'a Hourly> for WeatherReport<'a> {
    type Error = anyhow::Error;

    fn try_from(item: &'a Hourly) -> Result<Self> {
        let weather_description = item.weather.get(0).unwrap(); // TODO

        Ok(Self {
            name: None,
            description: &weather_description.description,
            icon: &weather_description.icon,
            temp: item.temp,
            temp_min: None,
            temp_max: None,
            feels_like: Some(item.feels_like),
            pressure: item.pressure,
            humidity: item.humidity,
            sunrise: None,
            sunset: None,
            visibility: Some(item.visibility),
            dt: item.dt,
        })
    }
}

impl<'a> TryFrom<&'a Daily> for WeatherReport<'a> {
    type Error = anyhow::Error;

    fn try_from(item: &'a Daily) -> Result<Self> {
        let weather_description = item.weather.get(0).unwrap(); // TODO
        let temperature = &item.temp;

        Ok(Self {
            name: None,
            description: &weather_description.description,
            icon: &weather_description.icon,
            temp: temperature.day,
            temp_min: None,
            temp_max: None,
            feels_like: None,
            pressure: item.pressure,
            humidity: item.humidity,
            sunrise: None,
            sunset: None,
            visibility: None,
            dt: item.dt,
        })
    }
}

pub struct WeatherController {
    current_report: Option<OpenWeather>,
    forecast: Option<OpenWeatherOnecall>,
}

impl WeatherController {
    pub fn new() -> Self {
        Self {
            current_report: None,
            forecast: None,
        }
    }

    pub fn refresh(&mut self, location: &str) -> Result<()> {
        self.current_report = self.fetch_current_weather(location).ok();

        if let Some(ref report) = self.current_report {
            self.forecast = self.fetch_forecast(&report.coord).ok()
        }

        Ok(())
    }

    pub fn current(&self) -> Option<WeatherReport> {
        let current_report = self.current_report.as_ref()?;

        current_report.try_into().ok()
    }

    pub fn hourly(&self, hour: usize) -> Option<WeatherReport> {
        let forecast = self.forecast.as_ref()?;
        let hour_report = forecast.hourly.get(hour)?;

        hour_report.try_into().ok()
    }

    pub fn daily(&self, day: usize) -> Option<WeatherReport> {
        let forecast = self.forecast.as_ref()?;
        let day_report = forecast.daily.get(day)?;

        day_report.try_into().ok()
    }

    fn fetch_current_weather(&self, location: &str) -> Result<OpenWeather> {
        let url = url::Url::parse(&format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&APPID={}&lang=de&units=metric",
            location, OPENWEATHER_API_KEY
        ))
        .expect("Invalid Url");
        let mut client = EspHttpClient::new_default()?;
        let response = client.get(url)?.submit()?;
        let response_reader = BufReader::new(StdIO(&response));

        let report: OpenWeather = serde_json::from_reader(response_reader)?;

        Ok(report)
    }

    fn fetch_forecast(&self, location: &Coord) -> Result<OpenWeatherOnecall> {
        let url = url::Url::parse(&format!(
            "https://api.openweathermap.org/data/2.5/onecall?lat={}&lon={}&APPID={}&lang=de&units=metric&exclude=current,minutely,alerts",
            location.lat, location.lon, OPENWEATHER_API_KEY
        ))
        .expect("Invalid Url");

        let mut client = EspHttpClient::new_default()?;
        let response = client.get(url)?.submit()?;
        let response_reader = BufReader::new(StdIO(&response));

        let report: OpenWeatherOnecall = serde_json::from_reader(response_reader)?;

        Ok(report)
    }
}

impl Default for WeatherController {
    fn default() -> Self {
        Self::new()
    }
}
