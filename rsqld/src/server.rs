use axum::{extract::Path, http::StatusCode, routing::post, Json, Router};
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio_postgres::types::Type;

#[derive(Deserialize)]
struct Payload {
    stmt: String,
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/:db_name", post(handle_request));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9876").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_request(
    Path(db_name): Path<String>,
    Json(payload): Json<Payload>,
) -> (StatusCode, Json<Value>) {
    // TODO: Change this to actually run the query against an rsql database
    let db_url = format!("postgres://postgres:postgres@localhost/{}", db_name);
    let (client, connection) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls)
        .await
        .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection failed {}", e);
        } else {
            println!("Connected to {}", db_url);
        }
    });

    let stmt = client.prepare(&payload.stmt).await.unwrap();
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
