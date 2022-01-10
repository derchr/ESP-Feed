use anyhow::Result;
use log::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub fn httpd() -> Result<esp_idf_svc::httpd::Server> {
    use anyhow::bail;
    use embedded_svc::httpd::registry::*;
    use embedded_svc::httpd::*;

    let form_handler = embedded_svc::httpd::Handler::new(
        "/form",
        embedded_svc::httpd::Method::Get,
        |req| -> Result<embedded_svc::httpd::Response> {
            Ok(embedded_svc::httpd::Response {
                status: 200,
                status_message: None,
                headers: BTreeMap::new(),
                body: embedded_svc::httpd::Body::Empty,
                new_session_state: None,
            })
        },
    );

    let server = esp_idf_svc::httpd::ServerRegistry::new()
        .at("/")
        .get(|_| {
            Ok(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/resources/settings.html"
            ))
            .into())
        })?
        .at("/simple")
        .get(|_| {
            Ok(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/resources/simple.html"
            ))
            .into())
        })?
        .at("/foo")
        .get(|_| bail!("Boo, something happened!"))?
        .at("/bar")
        .get(|_| {
            Response::new(403)
                .status_message("No permissions")
                .body("You have no permissions to access this page".into())
                .into()
        })?
        .at("/panic")
        .get(|_| panic!("User requested a panic!"))?
        .handler(form_handler)?
        .at("/favicon.ico")
        .get(|_| {
            Response::new(200)
                .content_type("image/x-icon")
                .body(embedded_svc::httpd::Body::Bytes(
                    include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/resources/favicon.ico"
                    ))
                    .to_vec(),
                ))
                .into()
        })?
        .at("/")
        .post(move |mut req| {
            let body = req.as_string().unwrap_or("".into());

            #[derive(Deserialize, Debug)]
            struct Form {
                name: String,
                ssid: String,
                auth: String,
            }

            let my_form: Form = serde_json::from_str(&body).unwrap();

            let resp = format!(
                "Header: {}<br>Body: {}<br>",
                req.header("User-Agent").unwrap_or_default(),
                body
            );

            info!("Send resp: {}, Form: {:?}", resp, my_form);

            Response::new(200).body(resp.into()).into()
        })?;

    server.start(&Default::default())
}
