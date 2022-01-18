use anyhow::Result;
use embedded_svc::{
    http::client::{Client, Request, Response},
    io::StdIO,
};
use esp_idf_svc::http::client::EspHttpClient;
use std::io::BufReader;
use std::io::Read;

pub struct WeatherPrimitive {}

struct WeatherState {
    // current
// forecast
}

pub fn current_weather() -> Result<WeatherPrimitive> {
    // let url = url::Url::parse("").expect("Invalid Url");
    // let mut client = EspHttpClient::new_default()?;
    // let response = client.get(url)?.submit()?;
    // let mut response_reader = BufReader::new(StdIO(&response));

    // let mut buf = String::new();
    // response_reader.read_to_string(&mut buf);

    // println!("{}", buf);

    Ok(WeatherPrimitive {})
}
