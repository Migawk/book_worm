use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};

pub fn check_similarity(text1: &str, text2: &str) -> f32 {
    let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
        .create_model()
        .unwrap();

    let sentences = [text1, text2];
    let embeddings: Vec<Vec<f32>> = model.encode(&sentences).unwrap();

    let embedding1 = &embeddings[0];
    let embedding2 = &embeddings[1];

    // Calculate the dot product
    let dot_product: f32 = embedding1
        .iter()
        .zip(embedding2.iter())
        .map(|(a, b)| a * b)
        .sum();

    // Calculate the magnitudes (norms)
    let norm1 = (embedding1.iter().map(|x| x * x).sum::<f32>()).sqrt();
    let norm2 = (embedding2.iter().map(|x| x * x).sum::<f32>()).sqrt();

    // Calculate cosine similarity
    let cosine_similarity = dot_product / (norm1 * norm2);

    cosine_similarity
}
