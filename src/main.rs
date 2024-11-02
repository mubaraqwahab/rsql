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
    let db_url = std::env::args()
        .nth(1)
        .unwrap_or(dotenvy::var("DATABASE_URL")?);

    let db_name = db_url.split("/").last().unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    println!("Connected to {db_url}");

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

        if input.is_empty() {
            continue;
        }

        let input_without_semi = if input.ends_with(";") {
            input[..input.len() - 1].to_string()
        } else {
            input
        };
        cmd_lines.push(input_without_semi);

        let cmd = cmd_lines.join(" ");
        cmd_lines.clear();

        if cmd == "exit" {
            break;
        } else {
            run_query(&cmd, &pool).await?;
        }
    }

    Ok(())
}

async fn run_query(query: &str, pool: &Pool<Postgres>) -> Result<(), Box<dyn std::error::Error>> {
    let records = sqlx::query(query).fetch_all(pool).await?;

    let mut table = pt::Table::new();

    if records.len() == 0 {
        println!("(0 rows)");
        return Ok(());
    }

    let field_names = records
        .first()
        .unwrap()
        .columns()
        .iter()
        .map(|c| c.name().to_string());
    table.add_row(field_names.into());

    for record in records {
        // let cells: Vec<String> = vec![];
        for column in record.columns() {
            dbg!(column.type_info());
            // column.type_info().clone_into(target);
        }
        // table.add_row(
        //     record
        //         .columns()
        //         .iter()
        //         .map(|c| record.get::<&str, &str>(c.name()))
        //         .into(),
        // );
    }

    table.printstd();

    Ok(())
}
