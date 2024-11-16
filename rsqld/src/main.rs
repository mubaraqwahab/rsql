use axum::{http::StatusCode, routing::post, Json, Router};
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio_postgres::types::Type;

#[derive(Deserialize)]
struct Payload {
    url: String,
    query: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", post(handle_request));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9876")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn col_to_json(col: &tokio_postgres::Column) -> Value {
    json!({
        "name": col.name(),
        "type": col.type_().name(),
    })
}

fn row_to_json(row: &tokio_postgres::Row) -> Value {
    let mut map = HashMap::new();

    for col in row.columns() {
        let idx = col.name();
        let value = match *col.type_() {
            Type::INT2 | Type::INT4 | Type::INT8 => json!(row.get::<_, i64>(idx)),
            Type::FLOAT4 | Type::FLOAT8 => json!(row.get::<_, f64>(idx)),
            Type::BOOL => json!(row.get::<_, bool>(idx)),
            Type::TIMESTAMP => {
                let datetime = row.get::<_, NaiveDateTime>(idx).and_utc();
                json!(datetime.to_rfc3339())
            }
            _ => json!(row.get::<_, String>(idx)),
        };

        map.insert(idx, value);
    }

    json!(map)
}

async fn handle_request(Json(payload): Json<Payload>) -> (StatusCode, Json<Value>) {
    let Payload { url, query } = payload;
    let (client, connection) = tokio_postgres::connect(&url, tokio_postgres::NoTls)
        .await
        .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let stmt = client.prepare(&query).await.unwrap();
    let cols: Vec<_> = stmt.columns().iter().map(col_to_json).collect();
    let rows: Vec<_> = client
        .query(&stmt, &[])
        .await
        .unwrap()
        .iter()
        .map(row_to_json)
        .collect();

    (StatusCode::OK, Json(json!({ "cols": cols, "rows": rows })))
}
