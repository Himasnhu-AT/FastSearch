mod database;
mod scoring;
mod util;

use std::time::SystemTime;

use axum::{extract::Query, routing::post, Json, Router};
use database::{
    pool::init_pool,
    queries::{get_document_count, get_keywords, group_by_websites},
};
use dotenv::dotenv;
use scoring::{proximity::get_proximities, tf_idf::get_tf_idf_scores};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use util::lemmetization::lemmatize;

#[derive(Deserialize)]
struct SearchQuery {
    query: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Website {
    pub title: String,
    pub description: String,
    pub url: String,
    pub rank: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QueryResult {
    execution_seconds: f32,
    results: Vec<Website>,
}

async fn query(
    db_pool: Pool<Postgres>,
    body: SearchQuery,
    document_count: i64,
) -> Json<QueryResult> {
    let now = SystemTime::now();

    let lemmatized_query = lemmatize(&body.query);
    let keywords = get_keywords(db_pool, &lemmatized_query)
        .await
        .expect("Failed to query keywords"); // TODO: Handle error properly

    let website_keywords = group_by_websites(keywords);

    if website_keywords.len() == 0 {
        return Json(QueryResult {
            execution_seconds: 0.,
            results: Vec::new(),
        });
    }

    let tf_idf_scores =
        get_tf_idf_scores(document_count, &lemmatized_query, &website_keywords).await;

    let proximity_scores = get_proximities(lemmatized_query, &website_keywords);
    let max_rank = website_keywords
        .values()
        .max_by(|(_, site_a), (_, site_b)| site_a.rank.cmp(&site_b.rank))
        .unwrap()
        .1
        .rank;

    let mut scores: Vec<(Website, f32)> = Vec::new();

    for (idx, (tf_idf, website)) in tf_idf_scores.iter().enumerate() {
        let proximity = proximity_scores[idx] * 0.2;
        let rank_score = (1. - (website.rank as f32 / max_rank as f32)) * 0.35;

        let final_score = (tf_idf + proximity + rank_score) / 3.;
        scores.push((website.clone(), final_score));
    }

    scores.sort_by(|(_, a_sim), (_, b_sim)| a_sim.partial_cmp(b_sim).unwrap());

    let results = scores
        .into_iter()
        .map(|(website, _)| website)
        .collect::<Vec<Website>>();

    let elapsed = now.elapsed().unwrap().as_secs_f32();

    Json(QueryResult {
        execution_seconds: elapsed,
        results: results,
    })
}

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to initialise .env");

    let pool = init_pool().await.expect("Failed to create pool");
    let document_count = get_document_count(pool.clone())
        .await
        .expect("Failed to get document count")
        .0;

    let router: Router<()> = Router::new().route(
        "/api/query",
        post(move |Json(body): Json<SearchQuery>| async move {
            query(pool, body, document_count).await
        }),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
