use dialoguer::theme::Theme;
use prettytable as pt;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;

pub struct DbConnection<'a> {
    url: &'a str,
    client: reqwest::Client,
}

impl<'a> DbConnection<'a> {
    pub fn bind(url: &'a str) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn execute(&self, query: &str) -> Result<QueryResult, Box<dyn std::error::Error>> {
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
pub struct QueryResult {
    rows: Vec<HashMap<String, serde_json::Value>>,
    columns: Vec<QueryResultColumn>,
}

#[derive(Debug, Deserialize)]
pub struct QueryResultColumn {
    name: String,
}

pub struct CliTheme;

impl Theme for CliTheme {
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

pub async fn run_query(
    query: &str,
    conn: &DbConnection<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
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
