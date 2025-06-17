use crate::semantic_router::SemanticRouter;
use crate::topics::{Topic, Utterance};

mod topics;
mod semantic_router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = SemanticRouter::new().await?;

    let bees_topic = Topic::new("bees");

    // create a vector of strings then iterate through the vector
    // and map them all to `Utterance` instances
    let bee_facts = vec![
        "Bees communicate with their hive mates through intricate dances that convey the location of nectar-rich flowers.",
        "A single bee can visit up to 5,000 flowers in a day, tirelessly collecting nectar and pollen.",
        "The queen bee can lay up to 2,000 eggs in a single day during peak season.",
    ].into_iter().map(|x| bees_topic.new_utterance(x)).collect::<Vec<Utterance>>();

    // embed utterances into Qdrant
    router.embed_utterances(bee_facts).await?;

    let bee_answer = router.query("how many flowers does a bee visit in a day?").await?;
    println!("Topic: {}", bee_answer.topic);

    // note that this query *should* error out as it's unrelated
    // in which case, we simply tell the user we can't help them
    match router.query("what is skibidi toilet").await {
        Ok(res) => println!("Unexpectedly found a topic: {}", res.topic),
        Err(_) => println!("Sorry, I can't help you with that.")
    };

    Ok(())

}
