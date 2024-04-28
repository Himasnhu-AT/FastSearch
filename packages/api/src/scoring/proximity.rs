use std::collections::HashMap;

use crate::database::queries::{Keyword, WebsiteKeywords};

pub fn get_proximities(
    lemmatized_query: Vec<String>,
    website_keywords: &WebsiteKeywords,
) -> Vec<f32> {
    let mut proximities = Vec::new();

    for (_, (keywords, _)) in website_keywords {
        let proximity = get_website_proximity(&lemmatized_query, &keywords);
        proximities.push(proximity);
    }

    proximities
}

pub fn get_website_proximity(lemmatized_query: &Vec<String>, keywords: &Vec<Keyword>) -> f32 {
    let mut clusters = Vec::new();
    let mut current_cluster = HashMap::new();
    let mut total_fullfillment = 0.;
    let mut word_positions = Vec::new();

    for (idx, keyword) in keywords.iter().enumerate() {
        word_positions.push(keyword.position);

        if current_cluster.contains_key(&keyword.word) {
            total_fullfillment += current_cluster.len() as f32 / lemmatized_query.len() as f32;
            clusters.push(current_cluster.clone());

            current_cluster.clear();
            current_cluster.insert(&keyword.word, keyword.position);

            continue;
        }

        current_cluster.insert(&keyword.word, keyword.position);
        if idx == keywords.len() {
            total_fullfillment += current_cluster.len() as f32 / lemmatized_query.len() as f32;
            clusters.push(current_cluster.clone());
        }
    }

    let mut fullfillment = 0.;
    if clusters.len() != 0 {
        fullfillment = total_fullfillment / clusters.len() as f32
    }

    let mut total_distance = 0;
    for i in 0..word_positions.len() - 1 {
        let position = word_positions[i];
        let next_position = word_positions[i + 1];

        let distance = next_position - position - 1;
        if distance > lemmatized_query.len() as i32 {
            continue;
        }

        total_distance += distance;
    }

    let mut proximity = 0.;
    if clusters.len() > 0 {
        proximity = 1. - (total_distance as f32 / keywords.len() as f32);
    }

    proximity * fullfillment
}
