//! TODO: Make types optional where needed.
//!
//! serde(flatten) results in a crash because it allocates too much. So sadly, we cannot use it.
//!
//! Some fields are commented out because at this point in time, there is no use for them.

use serde::{
    de::{Error, IgnoredAny, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::{fmt, marker::PhantomData};

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Coord {
    pub lat: f32,
    pub lon: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Description {
    // pub id: i32,
    // pub main: String,
    pub description: String,
    pub icon: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Main {
    // #[serde(flatten)]
    // pub temperature: Temperature,
    pub temp: f32,
    pub temp_min: f32,
    pub temp_max: f32,
    pub feels_like: f32,

    pub pressure: f32,
    pub humidity: f32,
    // pub sea_level: f32,
    // pub grnd_level: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Temperature {
    pub temp: f32,
    pub temp_min: f32,
    pub temp_max: f32,
    pub feels_like: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct TemperatureDaily {
    pub day: f32,
    pub min: f32,
    pub max: f32,
    pub night: f32,
    pub eve: f32,
    pub morn: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct FeelsLikeDaily {
    pub day: f32,
    pub night: f32,
    pub eve: f32,
    pub morn: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Wind {
    pub speed: f32,
    pub deg: f32,
    pub gust: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct WindOnecall {
    #[serde(rename = "wind_speed")]
    pub speed: f32,
    #[serde(rename = "wind_deg")]
    pub deg: f32,
    #[serde(rename = "wind_gust")]
    pub gust: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Precipitation {
    #[serde(rename = "1h")]
    pub one_hour: f32,
    #[serde(rename = "3h")]
    pub three_hour: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Clouds {
    pub all: i32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Sun {
    pub sunrise: i64,
    pub sunset: i64,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Moon {
    pub moonrise: i64,
    pub moonset: i64,
    pub moon_phase: f32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Sys {
    pub r#type: i32,
    pub id: i32,
    pub country: String,
    // #[serde(flatten)]
    // pub sun: Sun,
    pub sunrise: i64,
    pub sunset: i64,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Hourly {
    pub dt: i64,
    pub visibility: f32,
    // #[serde(flatten)]
    // pub temperature: Temperature,
    pub temp: f32,
    // pub temp_min: f32,
    // pub temp_max: f32,
    pub feels_like: f32,

    pub pressure: f32,
    pub humidity: f32,
    // #[serde(flatten)]
    // pub wind: WindOnecall,
    // pub clouds: i32,
    #[serde(deserialize_with = "deserialize_first")]
    pub weather: [Description; 1],
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Daily {
    pub dt: i64,
    // #[serde(flatten)]
    // pub sun: Sun,
    // #[serde(flatten)]
    // pub moon: Moon,
    pub temp: TemperatureDaily,
    // pub feels_like: FeelsLikeDaily,
    pub pressure: f32,
    pub humidity: f32,
    // #[serde(flatten)]
    // pub wind: WindOnecall,
    // pub clouds: i32,
    #[serde(deserialize_with = "deserialize_first")]
    pub weather: [Description; 1],
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct OpenWeather {
    pub coord: Coord,
    #[serde(deserialize_with = "deserialize_first")]
    pub weather: [Description; 1],
    pub base: String,
    pub main: Main,
    pub visibility: f32,
    pub wind: Wind,
    pub rain: Precipitation,
    pub snow: Precipitation,
    pub clouds: Clouds,
    pub dt: i64,
    pub sys: Sys,
    pub timezone: f32,
    pub id: i32,
    pub name: String,
    pub cod: i32,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct OpenWeatherOnecall {
    // #[serde(flatten)]
    // pub coord: Coord,
    #[serde(deserialize_with = "deserialize_first")]
    pub hourly: [Hourly; 6],
    #[serde(deserialize_with = "deserialize_first")]
    pub daily: [Daily; 6],
    // pub timezone: String,
    // pub timezone_offset: f32,
}

pub fn deserialize_first<'de, D, T, const N: usize>(deserializer: D) -> Result<[T; N], D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    deserializer.deserialize_seq(DeFirstVisitor(PhantomData))
}

struct DeFirstVisitor<T, const N: usize>(PhantomData<T>);
impl<'de, T, const N: usize> Visitor<'de> for DeFirstVisitor<T, N>
where
    T: Deserialize<'de>,
{
    type Value = [T; N];

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a sequence of at least {} elements", N)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let array = array_init::try_array_init(|i| {
            seq.next_element()?
                .ok_or_else(|| A::Error::invalid_length(i, &self))
        });

        while let Some(IgnoredAny) = seq.next_element()? {}

        array
    }
}
