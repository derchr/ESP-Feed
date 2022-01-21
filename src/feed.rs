use anyhow::{bail, Context, Result};
use embedded_svc::{
    http::client::{Client, Request},
    io::StdIO,
};
use esp_idf_svc::http::client::EspHttpClient;
use log::*;
use std::io::{BufRead, BufReader, Read};
use url::Url;

pub struct ScrollState {
    position: u32,
    forward: bool,
}

impl Default for ScrollState {
    fn default() -> Self {
        Self {
            position: 0,
            forward: true,
        }
    }
}
pub struct Feed {
    pub title: String,
    pub headlines: Vec<String>,
    // scroll position...
}
pub struct FeedController {
    feeds: Vec<Feed>,
    urls: Vec<Url>,
}

impl FeedController {
    pub fn new() -> Self {
        Self {
            feeds: Vec::new(),
            urls: Vec::new(),
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.feeds.clear();

        let mut client = EspHttpClient::new_default().context("Failed to create HTTP client.")?;

        for url in &self.urls {
            let response = client.get(url)?.submit()?;
            let mut response_reader = BufReader::new(StdIO(&response));

            match parse_rss_feed(&mut response_reader)
                .with_context(|| format!("Could not retrieve/parse feed {}", url))
            {
                Ok(feed) => {
                    info!("Got new feed: {}", feed.title);
                    for line in &feed.headlines {
                        info!("{}", line);
                    }

                    self.feeds.push(feed);
                }
                Err(e) => {
                    warn!("{:?}", e)
                }
            }
        }

        Ok(())
    }

    pub fn urls(&mut self) -> &mut Vec<Url> {
        &mut self.urls
    }

    pub fn feeds(&self) -> &[Feed] {
        &self.feeds
    }
}

fn parse_rss_feed(reader: &mut impl BufRead) -> Result<Feed> {
    let mut first_title = true;
    let mut title_follows = false;
    let mut title_count = 0;
    let mut title = String::new();
    let mut headlines = Vec::with_capacity(10);

    let mut buf = Vec::new();
    let mut parser = quick_xml::Reader::from_reader(reader);

    loop {
        use quick_xml::events::Event;

        match parser.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local_name = std::str::from_utf8(e.local_name())?;

                if local_name == "title" {
                    title_follows = true;
                }
            }
            Ok(Event::End(ref e)) => {
                let local_name = std::str::from_utf8(e.local_name())?;

                if local_name == "title" {
                    title_follows = false;
                }
            }
            Ok(Event::Text(e)) => {
                let content = e.unescape_and_decode(&parser)?;

                if first_title && title_follows {
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
            Err(e) => bail!(e),
            Ok(Event::Eof) => break,
            _ => (),
        }
    }

    Ok(Feed { title, headlines })
}
