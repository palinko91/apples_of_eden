use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collections {
    pub collections: Vec<CollectionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionItem {
    pub image: String,
    pub name: String,
    pub symbol: String,
    #[serde(rename = "totalItems")]
    pub total_items: Option<u32>,
    #[serde(rename = "onChainCollectionAddress")]
    pub on_chain_collection_address: Option<String>,
}

// Reading the collections.json and serialize for further processing
pub async fn read(filepath: &str) -> Result<Collections, Error> {
    let file = fs::read_to_string(filepath)
        .expect("The JSON containing the collection not found or can't be read");

    let collections: Collections =
        serde_json::from_str(&file).expect("JSON was not well-formatted");
    Ok(collections)
}

// Implementing the filter functions for the collection
impl Collections {
    // Purging out the empty collections
    pub async fn drop_empty_collections(mut self) -> Self {
        self.collections
            .retain(|item| item.total_items != None && item.total_items != Some(0));
        return self;
    }

    // Get the onchain address from NFT symbol
    pub async fn get_address(self, collection_symbol:&str) -> String {
        for item in self.collections {
            if item.symbol == collection_symbol.to_string() {
                return item.on_chain_collection_address.unwrap();
            }
        }
        return "The collection have no onchain address".to_string();
    }
}
