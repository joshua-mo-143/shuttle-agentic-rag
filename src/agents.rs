use std::sync::Arc;

use anyhow::Result;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs, CreateEmbeddingRequest,
    EmbeddingInput,
};
use async_openai::Embeddings;
use async_openai::{config::OpenAIConfig, Client as OpenAIClient};
use qdrant_client::prelude::{Payload, PointStruct, QdrantClient};
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::{
    with_payload_selector::SelectorOptions, SearchPoints, WithPayloadSelector,
};
use qdrant_client::qdrant::{CreateCollection, Distance, VectorParams, VectorsConfig};

use crate::files::File;

#[derive(Clone)]
pub struct MyAgent {
    openai_client: OpenAIClient<OpenAIConfig>,
    qdrant_client: Arc<QdrantClient>,
}

static COLLECTION: &str = "collection";

impl MyAgent {
    pub fn new(qdrant_client: QdrantClient) -> Self {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap();
        let config = OpenAIConfig::new().with_api_key(api_key);

        let openai_client = OpenAIClient::with_config(config);

        Self {
            openai_client,
            qdrant_client: Arc::new(qdrant_client),
        }
    }

    fn system_message(&self) -> String {
        "You're now a robot.".to_string()
    }

    pub async fn prompt(&self, prompt: &str) -> anyhow::Result<String> {
        let context = self.search_document(prompt.to_owned()).await?;
        let input = format!(
            "{prompt}

            Provided context:
            {}
            ",
            context
        );
        let res = self
            .openai_client
            .chat()
            .create(
                CreateChatCompletionRequestArgs::default()
                    .model("gpt-4o")
                    .messages(vec![
                        //First we add the system message to define what the Agent does
                        ChatCompletionRequestMessage::System(
                            ChatCompletionRequestSystemMessageArgs::default()
                                .content(&self.system_message())
                                .build()?,
                        ),
                        //Then we add our prompt
                        ChatCompletionRequestMessage::User(
                            ChatCompletionRequestUserMessageArgs::default()
                                .content(input)
                                .build()?,
                        ),
                    ])
                    .build()?,
            )
            .await
            .map(|res| {
                //We extract the first one
                res.choices[0].message.content.clone().unwrap()
            })?;

        println!("Retrieved result from prompt: {res}");

        Ok(res)
    }

    pub async fn embed_document(&self, file: File) -> Result<()> {
        let request = CreateEmbeddingRequest {
            model: "text-embedding-ada-002".to_string(),
            input: EmbeddingInput::StringArray(file.rows.clone()),
            user: None,
            dimensions: Some(1536),
            ..Default::default()
        };

        let embeddings_result = Embeddings::new(&self.openai_client).create(request).await?;

        for embedding in embeddings_result.data {
            let payload: Payload = serde_json::json!({
                "id": file.path.clone(),
                "content": file.contents,
                "rows": file.rows
            })
            .try_into()
            .unwrap();

            println!("Embedded: {}", file.path);

            let vec = embedding.embedding;

            let points = vec![PointStruct::new(
                uuid::Uuid::new_v4().to_string(),
                vec,
                payload,
            )];
            self.qdrant_client
                .upsert_points(COLLECTION, None, points, None)
                .await?;
        }
        Ok(())
    }

    pub async fn create_collection(&self) -> Result<()> {
        self.qdrant_client
            .create_collection(&CreateCollection {
                collection_name: COLLECTION.to_string(),
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: 1536,
                        distance: Distance::Cosine.into(),
                        hnsw_config: None,
                        quantization_config: None,
                        on_disk: None,
                        ..Default::default()
                    })),
                }),
                ..Default::default()
            })
            .await?;

        Ok(())
    }

    async fn search_document(&self, prompt: String) -> Result<String> {
        let request = CreateEmbeddingRequest {
            model: "text-embedding-ada-002".to_string(),
            input: EmbeddingInput::String(prompt),
            user: None,
            dimensions: Some(1536),
            ..Default::default()
        };

        let embeddings_result = Embeddings::new(&self.openai_client).create(request).await?;

        let embedding = &embeddings_result.data.first().unwrap().embedding;

        let payload_selector = WithPayloadSelector {
            selector_options: Some(SelectorOptions::Enable(true)),
        };

        let search_points = SearchPoints {
            collection_name: COLLECTION.to_string(),
            vector: embedding.to_owned(),
            limit: 1,
            with_payload: Some(payload_selector),
            ..Default::default()
        };

        let search_result = self.qdrant_client.search_points(&search_points).await?;
        let result = search_result.result.into_iter().next();

        match result {
            Some(res) => Ok(res.payload.get("contents").unwrap().to_string()),
            None => Err(anyhow::anyhow!("There were no results that matched :(")),
        }
    }
}
