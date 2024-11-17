use dialoguer::theme::Theme;
use prettytable as pt;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

pub struct DbConnection<'a> {
    pub db_name: &'a str,
    client: reqwest::Client,
}

impl<'a> DbConnection<'a> {
    pub fn bind(db_name: &'a str) -> Result<Self, std::io::Error> {
        // TODO: consider sending a request here to create the database if it doesn't yet exist
        Ok(Self {
            db_name,
            client: reqwest::Client::new(),
        })
    }

    pub async fn execute(&self, stmt: &str) -> Result<QueryResult, Box<dyn std::error::Error>> {
        let mut map = HashMap::new();
        map.insert("stmt", stmt);

        let db_url = format!("http://localhost:9876/{}", self.db_name);
        let resp = self.client.post(db_url).json(&map).send().await?;

        let query_result = resp.json().await?;
        Ok(query_result)
    }
}

#[derive(Debug, Deserialize)]
pub struct QueryResult {
    pub rows: Vec<QueryResultRow>,
    pub cols: Vec<QueryResultCol>,
}

impl QueryResult {
    pub fn print(&self) {
        if self.cols.is_empty() {
            return;
        }

        let mut table = pt::Table::new();
        table.set_format(*pt::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

        let col_names: Vec<_> = self.cols.iter().map(|c| &c.name).collect();
        table.set_titles(col_names.iter().into());

        for row in &self.rows {
            table.add_row(
                col_names
                    .iter()
                    .map(|&c| match row.get(c).unwrap() {
                        Value::String(v) => v.to_string(),
                        Value::Number(v) => v.to_string(),
                        Value::Bool(v) => v.to_string(),
                        Value::Null => "".to_string(),
                        v => panic!("Failed to parse json value {}", v),
                    })
                    .into(),
            );
        }

        table.printstd();
        println!("({} rows)", self.rows.len());
    }
}

pub type QueryResultRow = HashMap<String, Value>;

#[derive(Debug, Deserialize)]
pub struct QueryResultCol {
    pub name: String,
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
