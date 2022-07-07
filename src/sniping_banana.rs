use chrono::{
    Local,
    NaiveDateTime,
};
use reqwest::header::{
    HeaderMap,
    HeaderValue,
};
use serde::{
    Deserialize,
    Serialize,
};
use tokio::task::{
    JoinError,
    JoinHandle,
};

use crate::{
    config::{
        AuthConfig,
        ReserveReqs,
        RetryConfig,
    },
    utils::{
        parse_naive_date_time,
        SimpleError,
    },
};

#[derive(Debug)]
pub struct RustySnipingBanana {
    client: reqwest::Client,
}

impl RustySnipingBanana {
    pub fn new(auth_config: &AuthConfig) -> Self {
        Self {
            client: reqwest::Client::builder()
                .default_headers(create_headers(auth_config))
                .user_agent("python-requests/2.27.1")
                .build()
                .expect("could not build client"),
        }
    }

    pub async fn find_slots_with_retry(
        &self,
        reserve_reqs: &ReserveReqs,
        retry_config: &RetryConfig,
    ) -> Result<Vec<Slot>, Report> {
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_millis(retry_config.millisecs_between),
        );

        let mut attempts = Vec::<JoinHandle<_>>::new();
        for _ in 0..(retry_config.max_num_attempts) {
            interval.tick().await;

            let client_clone = self.client.clone();
            let reserve_reqs_clone = reserve_reqs.clone();
            attempts.push(tokio::spawn(async move {
                println!(
                    "Trying to find non-empty slots at time {}",
                    Local::now()
                );

                find_slots(client_clone, reserve_reqs_clone).await
            }));
        }

        let results = futures::future::join_all(attempts).await;
        let result_ok = |result: &Result<Result<_, _>, JoinError>| {
            result.is_ok() && result.as_ref().unwrap().is_ok()
        };
        if results.iter().any(result_ok) {
            return Ok(results
                .into_iter()
                .filter(result_ok)
                .next()
                .unwrap()
                .unwrap()
                .unwrap());
        }

        Err(Report::new(&results))
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct FindResponse {
    results: Results,
}

#[derive(Serialize, Deserialize, Debug)]
struct Results {
    venues: Vec<Venue>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Venue {
    slots: Vec<Slot>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Slot {
    pub size: SlotSize,
    pub date: SlotDates,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SlotSize {
    pub max: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SlotDates {
    #[serde(deserialize_with = "parse_naive_date_time")]
    pub start: NaiveDateTime,
}

#[derive(Debug)]
pub struct Report {
    num_async_errs: usize,
    example_async_err: Option<String>,
    num_req_errs: usize,
    example_req_err: Option<String>,
}

impl Report {
    pub fn new(
        results: &Vec<
            Result<
                Result<Vec<Slot>, Box<dyn std::error::Error + Send + Sync>>,
                JoinError,
            >,
        >
    ) -> Self {
        let async_err =
            |result: &&Result<Result<_, _>, JoinError>| result.is_err();
        let num_async_errs = results.iter().filter(async_err).count();
        let example_async_err: Option<String> = results
            .iter()
            .filter(async_err)
            .next()
            .map(|err| err.as_ref().unwrap_err().to_string());

        let req_err = |result: &&Result<Result<_, _>, JoinError>| {
            result.is_ok() && result.as_ref().unwrap().is_err()
        };
        let num_req_errs = results.iter().filter(req_err).count();
        let example_req_err: Option<String> =
            results.iter().filter(req_err).next().map(|err| {
                err.as_ref().unwrap().as_ref().unwrap_err().to_string()
            });

        Self {
            num_async_errs: num_async_errs,
            example_async_err: example_async_err,
            num_req_errs: num_req_errs,
            example_req_err: example_req_err,
        }
    }

    pub fn print(&self) {
        println!("Num async errors: {}", self.num_async_errs);
        if let Some(example) = &self.example_async_err {
            println!("Example async error: {}", example);
        }

        println!("Num request errors: {}", self.num_req_errs);
        if let Some(example) = &self.example_req_err {
            println!("Example request error: {}", example);
        }
    }
}

fn create_headers(auth_config: &AuthConfig) -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(
        "Authorization",
        HeaderValue::from_str(
            &format!("ResyAPI api_key=\"{}\"", &auth_config.api_key,)[..],
        )
        .expect("invalid api_key"),
    );
    headers.insert(
        "X-Resy-Auth-Token",
        HeaderValue::from_str(&auth_config.auth_token)
            .expect("invalid auth_token"),
    );

    headers
}

async fn find_slots(
    client: reqwest::Client,
    reserve_reqs: ReserveReqs,
) -> Result<Vec<Slot>, Box<dyn std::error::Error + Send + Sync>> {
    let response_text = client
        .get("https://api.resy.com/4/find")
        .query(&[
            ("lat", "0"),
            ("long", "0"),
            ("venue_id", &reserve_reqs.venue_id.to_string()[..]),
            ("day", &reserve_reqs.date.to_string()[..]),
            ("party_size", &reserve_reqs.party_size.to_string()[..]),
        ])
        .send()
        .await?
        .text()
        .await?;

    match serde_json::from_str::<FindResponse>(&response_text[..]) {
        Ok(mut find_response) => {
            let slots = find_response
                .results
                .venues
                .pop()
                .expect("empty venues when finding slots")
                .slots;

            if slots.is_empty() {
                return Err(
                    SimpleError::new("empty slots when finding slots").into()
                );
            }

            Ok(slots)
        },

        Err(err) => Err(SimpleError::new(
            &format!("{}: {}", err, response_text,)[..],
        )
        .into()),
    }
}
