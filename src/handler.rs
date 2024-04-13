use crate::smtpclient::SMTPClient;
use axum::{extract::ConnectInfo, extract::Request, http::Method, http::StatusCode};
use http_body_util::BodyExt;
use std::net::SocketAddr;
use tracing::info;

pub async fn receive_form(ConnectInfo(addr): ConnectInfo<SocketAddr>, req: Request) -> StatusCode {
    info!("Client connected {}", addr.ip().to_string());
    let (parts, body) = req.into_parts();
    if parts.method != Method::POST {
        return StatusCode::BAD_REQUEST;
    }
    let bytes = body.collect().await.unwrap().to_bytes().to_vec();
    let bytes: &[u8] = &bytes;

    let data = serde_urlencoded::from_bytes::<Vec<(String, String)>>(bytes).unwrap();

    let mut is_authed = false;
    let auth_key = data.iter().find(|x| x.0 == "key");
    if let Some(kv) = auth_key {
        let server_key = std::env::var("SERVER_KEY").expect("SERVER_KEY env var not found");
        is_authed = kv.1 == server_key;
    }
    if !is_authed {
        return StatusCode::FORBIDDEN;
    }

    let mut body = String::new();
    for kv in data.iter() {
        body.push_str(format!("{} -> {}\r\n", kv.0, kv.1).as_str());
    }

    let mut smtp = SMTPClient::new().await.unwrap();
    smtp.connect().await.expect("SMTP cannot connect");
    smtp.login().await.unwrap();
    smtp.send(body.as_str()).await.unwrap();
    info!("Email sent for {}", addr.ip().to_string());

    StatusCode::ACCEPTED
}
