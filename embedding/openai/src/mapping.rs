use greentic_dw_embedding::{
    EmbeddingError, EmbeddingErrorKind, EmbeddingRequest, EmbeddingResponse, EmbeddingResult,
    EmbeddingUsage,
};
use serde_json::{Value, json};

use crate::config::OpenAiEmbeddingConfig;

/// Builds the OpenAI `/embeddings` request body.
pub(crate) fn build_embeddings_request(
    config: &OpenAiEmbeddingConfig,
    request: &EmbeddingRequest,
) -> Value {
    let model = request
        .model
        .clone()
        .unwrap_or_else(|| config.model.clone());
    json!({ "model": model, "input": request.inputs })
}

/// Parses the OpenAI `/embeddings` response into the normalized shape.
/// Vectors are ordered by the response `index` field and truncated (never
/// padded) to `config.embedding_dim`.
pub(crate) fn parse_embeddings_response(
    config: &OpenAiEmbeddingConfig,
    response: &Value,
) -> EmbeddingResult<EmbeddingResponse> {
    let data = response
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| EmbeddingError::new(EmbeddingErrorKind::Provider, "missing data array"))?;

    let mut indexed: Vec<(usize, Vec<f32>)> = Vec::with_capacity(data.len());
    for item in data {
        let index = item.get("index").and_then(Value::as_u64).ok_or_else(|| {
            EmbeddingError::new(EmbeddingErrorKind::Provider, "embedding item missing index")
        })? as usize;
        let embedding = item
            .get("embedding")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                EmbeddingError::new(EmbeddingErrorKind::Provider, "missing embedding")
            })?;
        let mut vector: Vec<f32> = Vec::with_capacity(embedding.len());
        for value in embedding {
            let n = value.as_f64().ok_or_else(|| {
                EmbeddingError::new(
                    EmbeddingErrorKind::Provider,
                    "embedding value is not a number",
                )
            })?;
            vector.push(n as f32);
        }
        if vector.len() > config.embedding_dim {
            vector.truncate(config.embedding_dim);
        }
        indexed.push((index, vector));
    }
    indexed.sort_by_key(|(i, _)| *i);
    let vectors: Vec<Vec<f32>> = indexed.into_iter().map(|(_, v)| v).collect();

    let usage = response.get("usage").map(|u| EmbeddingUsage {
        prompt_tokens: u
            .get("prompt_tokens")
            .and_then(Value::as_u64)
            .map(|n| n as u32),
        total_tokens: u
            .get("total_tokens")
            .and_then(Value::as_u64)
            .map(|n| n as u32),
    });

    Ok(EmbeddingResponse {
        response_id: None,
        model: response
            .get("model")
            .and_then(Value::as_str)
            .map(str::to_string),
        dim: config.embedding_dim,
        vectors,
        usage,
        metadata: Value::Null,
    })
}
