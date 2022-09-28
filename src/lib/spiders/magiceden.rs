use crate::lib::error::Error;
use async_trait::async_trait;
use dotenv::dotenv;
use fantoccini::{Client, ClientBuilder};
use select::{document::Document, predicate::Name};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{create_dir_all, File};
use std::io::Write;
use tokio::sync::Mutex;

pub struct MagicSpider {
    webdriver_client: Mutex<Client>,
}

// Making the chromedriver instance
impl MagicSpider {
    pub async fn new() -> Result<Self, Error> {
        dotenv().expect("Failed to read .env file");
        let mut caps = serde_json::map::Map::new();
        // We can set the user-agent in the .env file to overwrite the automatic user-agent which carrying the headless attribute and raising bot protection
        let user_agent = format!(
            "--user-agent={}",
            &env::var("USER_AGENT")
                .expect("USER_AGENT must be set")
                .to_string()
        );
        let chrome_opts = serde_json::json!({ "args": ["--headless","--disable-gpu", user_agent] });
        caps.insert("goog:chromeOptions".to_string(), chrome_opts);
        let chromedriver_port = &env::var("CHROMEDRIVER_PORT")
            .expect("CHROMEDRIVER_PORT must be set")
            .to_string();
        let chromedriver_link = format!("http://localhost:{}", chromedriver_port);
        let webdriver_client = ClientBuilder::rustls()
            .capabilities(caps)
            .connect(&chromedriver_link)
            .await?;

        Ok(MagicSpider {
            webdriver_client: Mutex::new(webdriver_client),
        })
    }
}

// The source providing a JSON string and we storing in this struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionsItem {
    collections: String,
}

#[async_trait]
impl super::Spider for MagicSpider {
    type Item = CollectionsItem;

    fn name(&self) -> String {
        String::from("magicspider")
    }

    fn start_urls(&self) -> Vec<String> {
        vec!["https://api-mainnet.magiceden.io/all_collections?nowait=true/".to_string()]
    }

    async fn scrape(&self, url: String) -> Result<(Vec<Self::Item>, Vec<String>), Error> {
        let mut items = Vec::new();
        let html = {
            let webdriver = self.webdriver_client.lock().await;
            webdriver.goto(&url).await?;
            webdriver.source().await?
        };

        let document = Document::from(html.as_str());
        let data = document.select(Name("pre")).next();
        match data {
            Some(data) => items.push(CollectionsItem {
                collections: data.text().trim().to_string(),
            }),

            None => (),
        }
        // The function is designed to change to next page and continue scraping. We not using the next page so just returning a success sign, what we not even using.
        // But with this bypassing we don't have to change the crawler.rs because not sure we going to need the next page properties later
        let successful = vec!["Ok".to_owned()];

        Ok((items, successful))
    }

    // After the successful scrape we saving out all the info about the collection to a JSON
    async fn process(&self, info: Self::Item) -> Result<(), Error> {
        create_dir_all("./collections").unwrap_or_else(|e| panic!("Error creating dir: {}", e));
        let filename = "./collections/collections.json";
        let mut writer = File::create(&filename).unwrap();
        // Need to convert the data to bytes and then back to JSON to get rid of the \ escape sequences
        let data_in_bytes = (info.collections).as_bytes();
        let data_to_write: serde_json::Value = serde_json::from_slice(data_in_bytes).unwrap();
        writeln!(
            &mut writer,
            "{}",
            &serde_json::to_string_pretty(&data_to_write).unwrap()
        )
        .unwrap();
        println!("Magiceden Collections Updated!");
        Ok(())
    }
}
