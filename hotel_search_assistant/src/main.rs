mod hotel_search_tool;
mod location_search;
mod location_search_tool;
mod model;
mod utils;

use std::io;
use std::io::Write;
use crate::hotel_search_tool::HotelSearchTool;
use crate::location_search_tool::LocationSearchTool;
use dotenv::dotenv;
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::{Message, Prompt};
use rig::providers::openai::Client;
use chrono::Datelike;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let openai_client = Client::from_env();
    let current_year = chrono::Utc::now().year();

    let preamble = format!("You are a helpful assistant that finds hotels. \
        When providing hotel details, for each hotel, include its name, overall rating (with number of reviews), \
        price, key features, and its specific location or neighborhood within the city. \
        Do not invent information. If a direct booking link is not available, \
        mention the booking provider name if possible. You cannot provide specific positive or \
        negative comments from reviews, only the overall rating. \
         Keep responses concise unless more details are requested. \
         The current year is {}. If the user provides a date without a year, assume it's for {} if the date is in the future, \
         or {} if the date has already passed in {}. Provide dates in YYYY-MM-DD format.\
        If user didn't specify number of rooms, you should default to 1.
        ", current_year, current_year, current_year + 1, current_year);
    // The agent instance itself doesn't inherently store ongoing chat history across prompt calls.
    // We'll manage the history explicitly.
    let agent = openai_client
        .agent("gpt-4-turbo") // Recommended for better tool use in multi-turn
        .preamble(&preamble)
        .tool(LocationSearchTool)
        .tool(HotelSearchTool)
        .build();

    let mut chat_history: Vec<Message> = Vec::new();
    const MAX_CONVERSATION_TURNS: usize = 10; // Limit total conversation turns

    println!("Welcome to the Hotel Finder. Type 'exit' to quit.");

    loop {
        print!("\nUser: ");
        io::stdout().flush()?; // Ensure the prompt is displayed
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();

        if user_input.eq_ignore_ascii_case("exit") {
            println!("Goodbye!");
            break;
        }

        // Add user's message to history
        chat_history.push(Message::user(user_input));

        let mut agent_response_content = String::new(); // To accumulate agent's full textual response

        // Pass the entire history to the agent for context
        // Note: Rig's `agent.prompt()` takes a single `Message` or `&str`,
        // it doesn't directly take a `Vec<Message>` for ongoing history.
        // This implies Rig's Agent is more for single complex tasks.
        // For true persistent history, you would typically use `openai_client.chat().create()`
        // directly, managing the messages vector yourself.

        // However, for the purpose of demonstrating how to *try* to loop
        // with the agent and its internal multi-turn, we'll call prompt
        // with just the latest user input. The agent's `preamble` and tool descriptions
        // are its primary context. The `multi_turn` helps it make *internal* iterations.

        // If you need the LLM to remember *prior user/assistant dialogue*,
        // you would need to use a lower-level chat API call that accepts a `Vec<Message>`
        // and manage that `Vec<Message>` explicitly yourself, adding each user and assistant
        // message as it happens.

        // For now, let's assume `agent.prompt()` internally has some limited memory
        // for `multi_turn` but each call technically starts a new "thread" with preamble.
        // To truly persist memory across `prompt` calls, you'd likely rebuild the agent with history
        // or use `openai_client.chat().create` directly.

        // Let's revert to a simpler example for prompt call,
        // and acknowledge the limitation for full multi-turn conversational memory *between* calls.
        // The `multi_turn` config is for *one complex prompt*.

        println!("Agent thinking...");

        // Re-call prompt, but the "memory" across loops is not implicit with agent.prompt()
        // The LLM only "remembers" the history that is passed to it in the current prompt's context.
        // A true chatbot would build `Vec<Message>` and pass to `openai_client.chat().create`.
        let response_from_agent = agent
            .prompt(user_input) // Only sending the latest user input
            .multi_turn(5) // This multi_turn applies to internal tool-use chain for *this single prompt*
            .await?;

        println!("Agent: {}", response_from_agent);

        // This loop makes the interaction continuous, but the agent's memory
        // for *previous user questions/answers from prior loop iterations* is limited
        // unless explicitly passed back in the chat_history vector to a lower-level API.
        // For this `rig` Agent struct, the memory is primarily handled by the preamble
        // and the multi_turn config for a *single complex request*.
        // If you ask "What about the second hotel?" in the next turn, it might not remember the first search results.
    }

    Ok(())
}
