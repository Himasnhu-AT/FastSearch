use lazy_static::lazy_static;
use std::{collections::HashMap, fs};

lazy_static! {
    pub static ref LEMMATIZED_WORDS: HashMap<String, String> = get_lemmatized_words();
}

pub fn get_lemmatized_words() -> HashMap<String, String> {
    let raw_map = fs::read_to_string("./lemmatizedMap.json").unwrap();
    let lemmatized_words: HashMap<String, String> = serde_json::from_str(raw_map.as_str()).unwrap();
    lemmatized_words
}

pub fn lemmatize(text: &String) -> Vec<String> {
    text.to_lowercase()
        .chars()
        .filter(|c| !c.is_ascii_punctuation())
        .collect::<String>()
        .split_whitespace()
        .map(|s| {
            LEMMATIZED_WORDS
                .get(s)
                .unwrap_or(&s.to_string())
                .to_string()
        })
        .collect()
}
