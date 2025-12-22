use std::env;

pub struct Config {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn load() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .expect("PORT must be a number");

        Config { host, port }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
