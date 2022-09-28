use anyhow::Result;
use futures::prelude::*;
use solana_account_decoder::UiAccountEncoding::JsonParsed;
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
};
use solana_sdk::{clock::Slot, commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::str::FromStr;
use std::{env, sync::Arc, time::Duration};

mod lib;

pub use lib::crawler::Crawler;
pub use lib::filters::{read, Collections};
pub use lib::spiders::magiceden::MagicSpider;

// Solana RPC wss URL change node provider if needed
const URL: &str = "ws://api.mainnet-beta.solana.com";

#[tokio::main]
async fn main() -> Result<()> {
    // Create the scraper instance and makes the webscrape with the most up to date collections info and saves into a file
    let spider = MagicSpider::new().await?;
    let spider = Arc::new(spider);
    let crawler = Crawler::new(Duration::from_millis(200), 2, 500);
    crawler.run(spider).await;

    //Testing the filters
    let mut a: Collections = read("./src/collections/collections.json").await.unwrap();
    let a = a.drop_empty_collections().await;
    let b = a.get_address("odes").await;
    println!("{:?}",b);

    // Making an RPC client
    let rpc_client = PubsubClient::new(URL).await?;
    // Set the minimum slot that the request can be evaluated at.
    let slot: Slot = 1;
    let rpc_conf = RpcProgramAccountsConfig {
        // filter results using up to 4 filter objects; account must meet all filter criteria to be included
        // https://docs.solana.com/developing/clients/jsonrpc-api#filters
        // https://docs.rs/solana-client/latest/solana_client/rpc_filter/enum.RpcFilterType.html
        filters: None,
        account_config: RpcAccountInfoConfig {
            // Value usually Base64 or JsonParsed
            // https://docs.solana.com/developing/clients/jsonrpc-api#parsed-responses
            // https://docs.rs/solana-account-decoder/1.14.1/solana_account_decoder/enum.UiAccountEncoding.html
            encoding: Some(JsonParsed),
            //  limit the returned account data using the provided offset: <usize> and length: <usize> fields; only available for "base58", "base64" or "base64+zstd" encodings.
            data_slice: None,
            // Value can be called by the confirmed(), processed(), finalized() functions
            // https://docs.solana.com/developing/clients/jsonrpc-api#configuring-state-commitment
            // https://docs.rs/solana-sdk/1.14.1/solana_sdk/commitment_config/struct.CommitmentConfig.html
            commitment: Some(CommitmentConfig::confirmed()),
            min_context_slot: Some(slot),
        },
        // If have context slot then true, otherwise None or false not sure.
        with_context: Some(true),
    };

    // Set up the program address in this case magic eden's
    let me_address = Pubkey::from_str("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K").unwrap();

    // Creating the stream and only need one of the return value of the program_subscribe() function right now
    let (mut stream, _) = rpc_client
        .program_subscribe(&me_address, Some(rpc_conf))
        .await?;

    // Printing the events in the stream
    loop {
        let me_event = stream.next().await.unwrap();
        println!("{:?}", me_event);
        println!("---------------");
    }

    Ok(())
}
