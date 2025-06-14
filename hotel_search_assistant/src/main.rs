mod hotel_search_tool;
mod location_search;
mod model;

use dotenv::dotenv;
use rig::completion::Prompt;
use rig::providers::openai::Client;
use rig::client::{CompletionClient, ProviderClient};
use crate::hotel_search_tool::LocationSearchTool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    dotenv().ok();

    let openai_client = Client::from_env();

    let agent = openai_client
        .agent("gpt-4")
        .preamble("You are a helpful assistant that can find hotels for users.")
        .tool(LocationSearchTool)
        .build();

    let response = agent
        .prompt("Find me Hotel room in Vancouver, BC for 2 adults with check in date July 22 and checkout on July 27 2025.")
        .await?;
     println!("Agent response: \n{:?}", response);
    Ok(())
}
