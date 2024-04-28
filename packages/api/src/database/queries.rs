use std::collections::HashMap;

use sqlx::{Pool, Postgres};

use crate::Website;

#[derive(sqlx::FromRow)]
pub struct Keyword {
    pub url: String,
    pub word_count: i32,
    pub title: String,
    pub description: String,
    pub word: String,
    pub rank: i32,
    pub occurrences: i32,
    pub position: i32,
    pub documents_containing_word: i64,
}

pub async fn get_document_count(db_pool: Pool<Postgres>) -> Result<(i64,), sqlx::Error> {
    sqlx::query_as("SELECT COUNT(*) FROM websites;")
        .fetch_one(&db_pool)
        .await
}

pub async fn get_keywords(
    db_pool: Pool<Postgres>,
    query_words: &Vec<String>,
) -> Result<Vec<Keyword>, sqlx::Error> {
    let query_params = query_words
        .iter()
        .enumerate()
        .map(|(idx, _)| format!("${}::text", idx + 1))
        .collect::<Vec<String>>()
        .join(",");

    let sql_query = format!(
        "SELECT k.word, k.documents_containing_word, wk.occurrences, wk.position, w.url, w.word_count, w.title, w.description, w.rank
        FROM keywords k
        INNER JOIN website_keywords wk ON k.id = wk.keyword_id
        INNER JOIN websites w ON wk.website_id = w.id
        WHERE k.word in ({})
        ORDER BY wk.position
        ",
        query_params,
    );

    let mut query = sqlx::query_as(sql_query.as_str());

    for word in query_words {
        query = query.bind(word);
    }

    query.fetch_all(&db_pool).await
}

pub type WebsiteKeywords = HashMap<String, (Vec<Keyword>, Website)>;

pub fn group_by_websites(keywords: Vec<Keyword>) -> WebsiteKeywords {
    let mut website_keywords = HashMap::new();

    for keyword in keywords {
        // Should later probably use GROUP BY in the SQL query instead of this, if possible
        website_keywords
            .entry(keyword.url.clone())
            .or_insert((
                Vec::new(),
                Website {
                    url: keyword.url.clone(),
                    title: keyword.title.clone(),
                    description: keyword.description.clone(),
                    rank: keyword.rank.clone()
                },
            ))
            .0
            .push(keyword);
    }

    website_keywords
}
