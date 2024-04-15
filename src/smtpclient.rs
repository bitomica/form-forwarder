use crate::config::ServerConfig;
use base64::prelude::*;
use rustls_pki_types;
use std::io;
use std::sync::Arc;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::io::{AsyncWriteExt, BufWriter, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio_rustls::{client::TlsStream, rustls, TlsConnector};

#[derive(Debug)]
pub enum SMTPClientError {
    WrongInit,
    InvalidBuffer,
    CannotRead,
    HELOFailed,
    AUTHFailed,
    SendEmailFailed,
}

#[derive(Debug)]
pub struct SMTPClient {
    config: ServerConfig,
    reader: Option<BufReader<ReadHalf<TlsStream<TcpStream>>>>,
    writer: Option<BufWriter<WriteHalf<TlsStream<TcpStream>>>>,
    buffer: String,
}

impl SMTPClient {
    pub async fn new() -> Result<SMTPClient, SMTPClientError> {
        Ok(Self {
            config: ServerConfig::default(),
            reader: None,
            writer: None,
            buffer: String::new(),
        })
    }

    pub async fn connect(&mut self) -> Result<&Self, SMTPClientError> {
        let mut root_cert_store = rustls::RootCertStore::empty();
        root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        let connector = TlsConnector::from(Arc::new(config));
        let addr = format!("{}:{}", self.config.smtp_server, self.config.smtp_port);
        let stream = TcpStream::connect(addr.clone()).await.unwrap();

        let domain = rustls_pki_types::ServerName::try_from(self.config.smtp_domain.as_str())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))
            .unwrap()
            .to_owned();
        let stream = connector.connect(domain, stream).await.unwrap();

        let (reader, writer) = tokio::io::split(stream);
        let mut reader = BufReader::new(reader);
        if let Err(_) = reader.read_line(&mut self.buffer).await {
            return Err(SMTPClientError::InvalidBuffer);
        }

        self.reader = Some(reader);
        self.writer = Some(BufWriter::new(writer));

        //HELO
        let helo = format!("EHLO {}", self.config.smtp_domain.as_str());
        match self.send_cmd(helo.as_bytes()).await {
            Ok(_) => {
                while self.buffer.starts_with("250-") {
                    if let Err(_) = self.recv().await {
                        return Err(SMTPClientError::CannotRead);
                    }
                }
                return if self.buffer.starts_with("250") {
                    Ok(self)
                } else {
                    Err(SMTPClientError::HELOFailed)
                };
            }
            Err(_) => {
                return Err(SMTPClientError::HELOFailed);
            }
        }
    }

    async fn recv(&mut self) -> Result<usize, tokio::io::Error> {
        self.buffer.clear();
        match self.reader.as_mut() {
            Some(reader) => reader.read_line(&mut self.buffer).await,
            None => Err(tokio::io::Error::from(tokio::io::ErrorKind::InvalidInput)),
        }
    }

    async fn send_cmd(&mut self, data: &[u8]) -> Result<usize, tokio::io::Error> {
        match self.writer.as_mut() {
            Some(writer) => {
                writer.write_all(data).await?;
                writer.write_all(b"\r\n").await?;
                writer.flush().await?;
                self.buffer.clear();
                self.recv().await
            }
            None => Err(tokio::io::Error::from(tokio::io::ErrorKind::InvalidInput)),
        }
    }

    async fn send_data(&mut self, data: &[u8]) -> Result<usize, tokio::io::Error> {
        match self.writer.as_mut() {
            Some(writer) => {
                writer.write_all(data).await?;
                writer.write_all(b"\r\n.\r\n").await?;
                writer.write_all(b"\r\n.\r\n").await?;
                writer.flush().await?;
                self.buffer.clear();
                self.recv().await
            }
            None => Err(tokio::io::Error::from(tokio::io::ErrorKind::InvalidInput)),
        }
    }

    fn check_success(&self) -> Result<(), SMTPClientError> {
        if self.buffer.starts_with("250")
            || self.buffer.starts_with("334")
            || self.buffer.starts_with("354")
            || self.buffer.starts_with("235")
        {
            Ok(())
        } else {
            Err(SMTPClientError::AUTHFailed)
        }
    }

    pub async fn login(&mut self) -> Result<(), SMTPClientError> {
        if self.send_cmd(b"AUTH LOGIN").await.is_err() || self.check_success().is_err() {
            return Err(SMTPClientError::AUTHFailed);
        }

        if self
            .send_cmd(
                BASE64_STANDARD
                    .encode(self.config.smtp_user.as_str())
                    .as_bytes(),
            )
            .await
            .is_err()
            || self.check_success().is_err()
        {
            return Err(SMTPClientError::AUTHFailed);
        }

        if self
            .send_cmd(
                BASE64_STANDARD
                    .encode(self.config.smtp_pass.as_str())
                    .as_bytes(),
            )
            .await
            .is_err()
            || self.check_success().is_err()
        {
            return Err(SMTPClientError::AUTHFailed);
        }
        Ok(())
    }

    pub async fn send(&mut self, body: &str) -> Result<(), SMTPClientError> {
        let mail_from = format!("MAIL FROM:<{}>", self.config.sender);
        if self.send_cmd(mail_from.as_bytes()).await.is_err() || self.check_success().is_err() {
            return Err(SMTPClientError::SendEmailFailed);
        }

        let rcpt_to = format!("RCPT TO:<{}>", self.config.rcpt);
        if self.send_cmd(rcpt_to.as_bytes()).await.is_err() || self.check_success().is_err() {
            return Err(SMTPClientError::SendEmailFailed);
        }

        if self.send_cmd(b"DATA").await.is_err() || self.check_success().is_err() {
            return Err(SMTPClientError::SendEmailFailed);
        }

        let content = format!("Subject: {}\r\n\r\n{body}", self.config.subject);
        if self.send_data(content.as_bytes()).await.is_err() || self.check_success().is_err() {
            return Err(SMTPClientError::SendEmailFailed);
        }

        Ok(())
    }
}
