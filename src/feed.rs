use anyhow::{bail, Result};
use embedded_svc::{
    http::client::{Client, Request},
    io::StdIO,
};
use esp_idf_svc::http::client::EspHttpClient;
use log::*;
use std::io::BufReader;
use url::Url;

pub struct Feed {
    pub title: String,
    pub headlines: Vec<String>,
}

pub fn rss_feed(url: &Url) -> Result<Feed> {
    let mut first_title = true;
    let mut title_follows = false;
    let mut title_count = 0;
    let mut title = String::new();
    let mut headlines = Vec::new();

    let config = xml::ParserConfig::new().trim_whitespace(true);

    let mut http_client = EspHttpClient::new_default()?;
    let request = http_client.get(url)?.submit()?;

    let mut request_reader = BufReader::new(StdIO(&request));

    let parser = xml::reader::EventReader::new_with_config(&mut request_reader, config);

    for e in parser {
        match e {
            Ok(xml::reader::XmlEvent::StartElement { name, .. }) => {
                if name.local_name == "title" {
                    title_follows = true;
                }
            }
            Ok(xml::reader::XmlEvent::EndElement { name }) => {
                if name.local_name == "title" {
                    title_follows = false;
                }
            }
            Ok(xml::reader::XmlEvent::Characters(content)) => {
                if first_title {
                    title = content;
                    first_title = false;
                    continue;
                }

                if title_follows {
                    title_count += 1;
                    headlines.push(content);
                }

                if title_count == 10 {
                    break;
                }
            }
            Err(e) => {
                warn!("Parse error: {}", e);
                bail!("Parse error: {}", e);
            }
            _ => {}
        }
    }

    Ok(Feed { title, headlines })
}
