use dialoguer::{theme::Theme, BasicHistory, Input};
use prettytable as pt;
use sqlx::{postgres::PgPoolOptions, Pool};
use sqlx::{Column, Postgres, Row};
use std::{fmt, vec};

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
    let database_url = std::env::args()
        .nth(1)
        .unwrap_or(dotenvy::var("DATABASE_URL")?);

    let db_name = database_url.split("/").last().unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Connected to {database_url}");

    let mut history = BasicHistory::new().no_duplicates(true);
    let mut cmd_lines: Vec<String> = vec![];

    loop {
        let prompt = if cmd_lines.len() == 0 {
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

        if !input.is_empty() {
            cmd_lines.push(input.clone());
        }

        if input == "exit" || input == "exit;" {
            break;
        } else if input.ends_with(";") {
            let cmd = cmd_lines.join("\n");
            cmd_lines.clear();
            run_query(&cmd, &pool).await?;
        }
    }

    Ok(())
}

async fn run_query(query: &str, pool: &Pool<Postgres>) -> Result<(), Box<dyn std::error::Error>> {
    let rows = sqlx::query(query).fetch_all(pool).await?;

    let mut table = pt::Table::new();

    let headers: Vec<&str> = rows
        .iter()
        .next()
        .unwrap()
        .columns()
        .iter()
        .map(|c| c.name())
        .collect();

    table.add_row(headers.into());

    table.printstd();

    Ok(())
}
