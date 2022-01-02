use anyhow::Result;
use log::*;

use crate::https_client::*;

pub struct Feed {
    pub title: String,
    pub headlines: Vec<String>,
}

pub fn rss_feed() -> Result<Feed> {
    let url = url::Url::parse("https://www.tagesschau.de/newsticker.rdf").expect("Invalid Url");
    let response = https_request(&url)?;

    // Don't parse the HTTP response stuff...
    let index = response.iter().position(|x| *x == b'<').unwrap();

    let parser = xml::reader::EventReader::new(&response[index..]);
    let mut title = String::new();
    let mut headlines = Vec::new();
    let mut first_title = true;
    let mut title_follows = false;
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
                    headlines.push(content);
                }
            }
            Err(e) => {
                warn!("Parse error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(Feed { title, headlines })
}
