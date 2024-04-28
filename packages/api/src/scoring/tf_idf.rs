use crate::{database::queries::WebsiteKeywords, Website};
use std::collections::HashMap;

pub async fn get_tf_idf_scores(
    document_count: i64,
    lemmatized_query: &Vec<String>,
    website_keywords: &WebsiteKeywords,
) -> Vec<(f32, Website)> {
    let mut query_term_tfs: HashMap<String, f32> = HashMap::new();
    let mut query_word_occurences: HashMap<String, i32> = HashMap::new();

    for word in lemmatized_query {
        *query_word_occurences.entry(word.to_string()).or_insert(0) += 1;
    }

    for word in lemmatized_query {
        let tf = query_word_occurences[word] as f32 / lemmatized_query.len() as f32;
        query_term_tfs.insert(word.clone(), tf);
    }

    let mut website_similarities: Vec<(f32, Website)> = Vec::new();

    for (_, (keywords, website)) in website_keywords {
        let mut query_vector_sum = 0.;
        let mut document_vector_sum = 0.;

        let mut dot_product = 0.;

        for keyword in keywords {
            let tf = keyword.occurrences as f32 / keyword.word_count as f32;
            let idf = 1. + (document_count as f32 / keyword.documents_containing_word as f32).ln();
            let tf_idf = tf * idf;

            let query_tf_idf = query_term_tfs[&keyword.word] * idf;
            query_vector_sum += query_tf_idf.powi(2);
            document_vector_sum += tf_idf.powi(2);

            dot_product += query_tf_idf * tf_idf;
        }

        let query_vector = query_vector_sum.sqrt();
        let document_vector = document_vector_sum.sqrt();
        let similarity = dot_product / (query_vector * document_vector);
        website_similarities.push((similarity, website.clone()));
    }

    website_similarities
}
