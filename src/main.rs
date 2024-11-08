use dialoguer::{theme::Theme, BasicHistory, Input};
use prettytable as pt;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;

struct Connection<'a> {
    url: &'a str,
    client: reqwest::Client,
}

impl<'a> Connection<'a> {
    fn new(url: &'a str) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
        }
    }

    async fn execute(&self, query: &str) -> Result<QueryResult, Box<dyn std::error::Error>> {
        let mut map = HashMap::new();
        map.insert("query", query);
        map.insert("url", self.url);

        let resp = self
            .client
            .post("http://localhost:9876")
            .json(&map)
            .send()
            .await?;

        let query_result: QueryResult = resp.json().await?;
        Ok(query_result)
    }
}

#[derive(Debug, Deserialize)]
struct QueryResult {
    rows: Vec<HashMap<String, serde_json::Value>>,
    columns: Vec<QueryResultColumn>,
}

#[derive(Debug, Deserialize)]
struct QueryResultColumn {
    name: String,
}

struct MyTheme;

impl Theme for MyTheme {
    /// Formats an input prompt.
    fn format_input_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<&str>,
    ) -> fmt::Result {
        match default {
            Some(default) if prompt.is_empty() => write!(f, "[{}] ", default),
            Some(default) => write!(f, "{} [{}] ", prompt, default),
            None => write!(f, "{} ", prompt),
        }
    }

    /// Formats an input prompt after selection.
    #[inline]
    fn format_input_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        write!(f, "{} {}", prompt, sel)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = std::env::args()
        .nth(1)
        .unwrap_or(dotenvy::var("DATABASE_URL")?);
    let db_name = db_url.split("/").last().unwrap();

    let conn = Connection::new(&db_url);
    println!("Connected to database {db_name}");

    let mut history = BasicHistory::new().no_duplicates(true);
    let mut cmd_lines: Vec<String> = vec![];

    let exit_cmd_re = Regex::new(r"^exit\s*;*$").unwrap();
    let trailing_semi_re = Regex::new(";*$").unwrap();

    loop {
        let prompt = if cmd_lines.is_empty() {
            db_name.to_string() + "=#"
        } else {
            db_name.to_string() + "-#"
        };

        let input = Input::<String>::with_theme(&MyTheme)
            .with_prompt(prompt)
            .history_with(&mut history)
            .allow_empty(true)
            .interact_text()
            .unwrap();

        let input = input.trim().to_lowercase();

        if input.is_empty() {
            continue;
        } else if exit_cmd_re.is_match(&input) {
            break;
        } else if input.ends_with(";") {
            let input_without_semi = trailing_semi_re.replace(&input, "").into_owned();
            cmd_lines.push(input_without_semi);

            let cmd = cmd_lines.join(" ");
            cmd_lines.clear();

            let _ = run_query(&cmd, &conn)
                .await
                .inspect_err(|e| eprintln!("failed to execute query {e:?}"));
        } else {
            cmd_lines.push(input);
        }
    }

    Ok(())
}

async fn run_query(query: &str, conn: &Connection<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let QueryResult { rows, columns } = conn.execute(query).await?;

    if columns.is_empty() {
        return Ok(());
    }

    let mut table = pt::Table::new();
    table.set_format(*pt::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    let column_names: Vec<&String> = columns.iter().map(|c| &c.name).collect();
    table.set_titles(column_names.iter().into());

    for row in &rows {
        table.add_row(
            column_names
                .iter()
                .map(|&c| match row.get(c).unwrap() {
                    serde_json::Value::String(v) => v.clone(),
                    serde_json::Value::Number(v) => v.to_string(),
                    serde_json::Value::Bool(v) => v.to_string(),
                    serde_json::Value::Null => String::from(""),
                    v => panic!("Failed to parse json value {}", v),
                })
                .into(),
        );
    }

    table.printstd();
    println!("({} rows)", rows.len());

    Ok(())
}
