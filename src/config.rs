use std::{
    fs::File,
    io::BufReader,
    path::Path,
};

use chrono::{
    NaiveDate,
    NaiveDateTime,
    NaiveTime,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::utils::parse_naive_date_time;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub auth: AuthConfig,
    pub reqs: ReserveReqs,
    #[serde(deserialize_with = "parse_naive_date_time")]
    pub wait_till: NaiveDateTime,
    pub retry: RetryConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthConfig {
    pub api_key: String,
    pub auth_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReserveReqs {
    pub venue_id: i32,
    pub date: NaiveDate,
    pub earliest_time: NaiveTime,
    pub party_size: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RetryConfig {
    pub millisecs_between: u64,
    pub max_num_attempts: usize,
}

pub fn read_config<P: AsRef<Path>>(
    path: P
) -> Result<Config, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    serde_json::from_reader(reader).map_err(|err| err.into())
}
