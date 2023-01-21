use futures::{stream, StreamExt};
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::{
    io::{self, BufRead, BufReader},
    path::Path,
};

const CONCURRENT_REQUESTS: usize = 16;

#[derive(Serialize, Deserialize, Debug)]
struct Relay {
    contact: String,
    description: String,
    name: String,
    software: String,
    supported_nips: Vec<i32>,
    version: String,
}

fn load_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    BufReader::new(File::open(filename)?).lines().collect()
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let file = "./relays.yaml";
    let relays = load_file(file).unwrap();
    // Nip you are looking for on relays
    let nip = 33;

    let client = reqwest::Client::new();
    let bodies = stream::iter(relays)
        .map(|url| {
            let client = &client;
            async move {
                let resp = client
                    .get(&url)
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;
                let text = resp.text().await?;

                let r: Result<(String, String), reqwest::Error> = Ok((url, text));

                r
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    bodies
        .for_each(|b| async {
            if let Ok((url, json)) = b {
                let data: Result<Relay, serde_json::Error> = serde_json::from_str(&json);
                if let Ok(json) = data {
                    for n in &json.supported_nips {
                        if n == &nip {
                            println!("{} Supports nip{nip}", url);
                        }
                    }
                }
            }
        })
        .await;

    Ok(())
}
