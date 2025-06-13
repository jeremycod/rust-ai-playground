mod flight_search_tool;
use crate::flight_search_tool::{FlightOption, FlightSearchTool};
use dotenv::dotenv;
use rig::completion::Prompt;
use rig::providers::openai::Client;
use rig::client::{CompletionClient, ProviderClient};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let openai_client = Client::from_env();

    let agent = openai_client
        .agent("gpt-4")
        .preamble("You are a helpful assistant that can find flights for users.")
        .tool(FlightSearchTool)
        .build();

    let response = agent
        .prompt("Find me cheapest flights from Vancouver (YVR) to Belgrade (BEG) between on July 15 2025.")
        .await?;

    println!("Agent response: \n{}", response);
    Ok(())
}
