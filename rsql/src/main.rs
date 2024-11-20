use clap::Parser;
use dialoguer::{BasicHistory, Input};
use regex::Regex;
use rsql::*;

/// The interactive terminal for the rsql database system.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The name of the database to connect to
    db_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let db_name = args.db_name;

    // TODO: if the rsqld server is unreachable, terminate here.
    let conn =
        DbConnection::bind(&db_name).expect(&format!("failed to connect to database {db_name}"));

    let pkg_version = env!("CARGO_PKG_VERSION");
    println!("rsql {}\n", pkg_version);

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

        let input = Input::<String>::with_theme(&CliTheme)
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

            match conn.execute(&cmd).await {
                Ok(result) => result.print(),
                Err(e) => eprintln!("failed to execute query {e:?}"),
            }
        } else {
            cmd_lines.push(input);
        }
    }

    Ok(())
}
