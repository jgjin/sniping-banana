use chrono::{
    Local,
    NaiveDateTime,
};
use serde::{
    de,
    Deserialize,
    Deserializer,
};
use tokio::time::sleep;

#[derive(Debug)]
pub struct SimpleError {
    message: String,
}

impl SimpleError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for SimpleError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SimpleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub fn parse_naive_date_time<'de, D: Deserializer<'de>>(
    deserializer: D
) -> Result<NaiveDateTime, D::Error> {
    let s: String = Deserialize::deserialize(deserializer)?;

    NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
        .map_err(de::Error::custom)
}

pub async fn sleep_till(deadline: &NaiveDateTime) {
    let now = Local::now().naive_local();
    if *deadline > now {
        let duration = (*deadline - now)
            .to_std()
            .expect("sleep duration out of range");

        println!("sleeping {:?}", duration);
        sleep(duration).await
    }
}
