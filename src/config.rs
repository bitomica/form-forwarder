#[derive(Debug)]
pub struct ServerConfig {
    pub port: u32,
    pub smtp_server: String,
    pub smtp_port: u32,
    pub smtp_domain: String,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub sender: String,
    pub rcpt: String,
    pub subject: String,
}

impl ServerConfig {
    pub fn default() -> ServerConfig {
        ServerConfig {
            port: std::env::var("SERVER_PORT")
                .expect("SERVER_PORT env var must be set")
                .parse::<u32>()
                .unwrap(),
            smtp_server: std::env::var("SMTP_SERVER").expect("SMTP_SERVER env var must be set"),
            smtp_port: std::env::var("SMTP_PORT")
                .expect("SMTP_PORT env var must be set")
                .parse::<u32>()
                .unwrap(),
            smtp_domain: std::env::var("SMTP_DOMAIN").expect("SMTP_DOMAIN env var must be set"),
            smtp_user: std::env::var("SMTP_USER").expect("SMTP_USER env var must be set"),
            smtp_pass: std::env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD env var must be set"),
            sender: std::env::var("EMAIL_SENDER").expect("EMAIL_SENDER env var must be set"),
            rcpt: std::env::var("EMAIL_RCPT").expect("EMAIL_RCPT env var must be set"),
            subject: std::env::var("EMAIL_SUBJECT").expect("EMAIL_SUBJECT env var must be set"),
        }
    }
}
