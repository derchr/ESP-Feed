//! Fetches the stock info and provides the StockController.

use crate::datetime;
use anyhow::Result;
use embedded_plots::curve::PlotPoint;
use embedded_svc::{
    http::client::{Client, Request},
    io::StdIO,
};
// use esp_idf_svc::http::client::EspHttpClient;
use itertools::Itertools;
use serde::{
    de::{IgnoredAny, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::{collections::HashMap, fmt, hash::Hash, io::BufReader, marker::PhantomData};

const ALPHAVANTAGE_API_KEY: &str = env!("ALPHAVANTAGE_API_KEY");
const NUM_ENTRIES: usize = 48;

pub struct StockController(Option<[PlotPoint; NUM_ENTRIES]>);

impl Default for StockController {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct Stock {
    #[serde(rename = "Time Series (Daily)")]
    #[serde(deserialize_with = "deserialize_first")]
    daily: HashMap<String, Daily>,
}

#[derive(Debug, Deserialize)]
struct Daily {
    #[serde(rename = "4. close")]
    close: String,
}

fn deserialize_first<'de, D, K, V>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
{
    deserializer.deserialize_map(DeFirstVisitor(PhantomData))
}

struct DeFirstVisitor<K, V>(PhantomData<fn() -> HashMap<K, V>>);

impl<'de, K, V> Visitor<'de> for DeFirstVisitor<K, V>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
{
    type Value = HashMap<K, V>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a map of at least {} entries", NUM_ENTRIES)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut hash_map: Self::Value = HashMap::with_capacity(NUM_ENTRIES);

        let mut i = 0usize;

        while let Some((key, value)) = map.next_entry()? {
            hash_map.insert(key, value);

            i += 1;
            if i >= NUM_ENTRIES {
                break;
            }
        }

        while let Some((IgnoredAny, IgnoredAny)) = map.next_entry()? {}

        Ok(hash_map)
    }
}

impl StockController {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn stock_data(&self) -> Option<&[PlotPoint]> {
        if let Some(ref data) = self.0 {
            Some(data.as_slice())
        } else {
            None
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        let url = url::Url::parse(&format!(
            "https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&symbol=IBM&apikey={}",
            ALPHAVANTAGE_API_KEY
        ))
        .expect("Invalid Url");

        let format = time::format_description::parse("[year]-[month]-[day]").unwrap();

        let mut client = esp_idf_svc::http::client::EspHttpClient::new_default()?;
        let response = client.get(url)?.submit()?;
        let response_reader = BufReader::new(StdIO(&response));

        let stock: Stock = serde_json::from_reader(response_reader)?;

        let data_iter = stock
            .daily
            .into_iter()
            .map(|(date, daily)| {
                let value = daily.close.parse::<f32>().unwrap();

                let mut parsed = time::parsing::Parsed::new();
                parsed
                    .parse_items(date.as_bytes(), format.as_slice())
                    .unwrap();

                let date = time::Date::from_calendar_date(
                    parsed.year().unwrap(),
                    parsed.month().unwrap(),
                    parsed.day().unwrap().get(),
                )
                .unwrap();

                let current = datetime::get_datetime().unwrap().date();

                ((date - current).whole_days(), value)
            })
            .sorted_by_key(|&(date, _)| date)
            .map(|(date, value)| PlotPoint {
                x: date as i32,
                y: value as _,
            });

        self.0 = Some(array_init::from_iter(data_iter).unwrap());

        Ok(())
    }
}
