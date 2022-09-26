use crate::lib::error::Error;
use async_trait::async_trait;
use fantoccini::{Client, ClientBuilder};
use select::{
    document::Document,
    predicate::Name,
};
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::env;
use dotenv::dotenv;

pub struct MagicSpider {
    webdriver_client: Mutex<Client>,
}

impl MagicSpider {
    pub async fn new() -> Result<Self, Error> {
        dotenv().expect("Failed to read .env file");
        let mut caps = serde_json::map::Map::new();
        let user_agent = format!("--user-agent={}",&env::var("USER_AGENT").expect("USER_AGENT must be set").to_string());
        let chrome_opts = serde_json::json!({ "args": ["--headless","--disable-gpu", user_agent] });
        caps.insert("goog:chromeOptions".to_string(), chrome_opts);
        let chromedriver_port = &env::var("CHROMEDRIVER_PORT").expect("CHROMEDRIVER_PORT must be set").to_string();
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionsItem {
    collections: String,
}

#[async_trait]
impl super::Spider for MagicSpider {
    type Item = CollectionsItem;

    fn name(&self) -> String {
        String::from("quotes")
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
            Some(data) => 
                items.push(CollectionsItem {collections: data.text().trim().to_string()}),
            
            None => ()
        }
        let successful = vec!["Ok".to_owned()];

        Ok((items, successful))
    }

    async fn process(&self, info: Self::Item) -> Result<(), Error>{
        create_dir_all("./collections").unwrap_or_else(|e| panic!("Error creating dir: {}", e));
        let filename = "./collections/collections.json";
        let mut writer = File::create(&filename).unwrap();
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
