#![feature(async_closure)]

mod config;
mod sniping_banana;
mod utils;

use config::read_config;
use sniping_banana::RustySnipingBanana;
use utils::sleep_till;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = read_config("config.json")?;

    let banana = RustySnipingBanana::new(&config.auth);

    sleep_till(&config.wait_till).await;

    match banana
        .find_slots_with_retry(&config.reqs, &config.retry)
        .await
    {
        Ok(slots) => {
            println!("{:?}", slots);
        },
        Err(report) => {
            report.print();
        },
    }

    Ok(())
}
