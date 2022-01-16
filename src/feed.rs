use anyhow::{bail, Context, Result};
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

        for url in self.urls.clone() {
            match self
                .rss_feed(&url)
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

    fn rss_feed(&mut self, url: &Url) -> Result<Feed> {
        let mut first_title = true;
        let mut title_follows = false;
        let mut title_count = 0;
        let mut title = String::new();
        let mut headlines = Vec::with_capacity(10);

        let mut client = EspHttpClient::new_default().context("Failed to create HTTP client.")?;
        let request = client.get(url)?.submit()?;

        let request_reader = BufReader::new(StdIO(&request));

        let mut buf = Vec::new();
        let mut parser = quick_xml::Reader::from_reader(request_reader);

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
}
