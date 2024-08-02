use ansi_term::{Style, Color};
#[allow(unused_imports)]
use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use console::Emoji;
use dirs;
use tabled::{builder::Builder, settings::Style as TabledStyle};
use crate::{
    db::Database,
    streaks::{Frequency, Streak},
    tui,
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[clap(short, long, default_value = "")]
    database_url: String,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "List all streaks", long_about = None, short_flag = 'l')]
    List,
    #[command(about = "Create a new streak", long_about = None, short_flag = 'a')]
    Add {
        #[clap(short, long, value_enum)]
        frequency: Frequency,

        #[clap(short, long)]
        name: String,
    },
    #[command(about = "Get a single streak", long_about = None, short_flag='g')]
    Get { idx: u32 },
    #[command(about = "Check in to a streak", long_about = None, short_flag = 'c')]
    CheckIn { idx: u32 },
    #[command(about = "Remove a streak", long_about = None, short_flag = 'r')]
    Remove { idx: u32 },
    #[command(about = "Switch to TUI", long_about = None, short_flag = 't')]
    Tui,
}

/// Create a new daily streak item
fn new_daily(name: String, db: &mut Database) -> Result<Streak, Box<dyn std::error::Error>> {
    let streak = Streak::new_daily(name);
    db.streaks.lock().unwrap().push(streak.clone());
    db.save()?;
    Ok(streak)
}

/// Create a new weekly streak item
fn new_weekly(name: String, db: &mut Database) -> Result<Streak, Box<dyn std::error::Error>> {
    let streak = Streak::new_weekly(name);
    db.streaks.lock().unwrap().push(streak.clone());
    db.save()?;
    Ok(streak)
}

/// Get all streaks
fn get_all(mut db: Database) -> Vec<Streak> {
    match db.get_all() {
        Some(streaks) => streaks.clone(),
        None => Vec::<Streak>::new()
    }
}

/// Get one single streak
fn get_one(db: &mut Database, idx: u32) -> Option<Streak>{
    if let Some(streak) = db.get_one(idx) {
        return Some(streak.clone())
    }
    None
}

/// Check in to a streak today
fn checkin(db: &mut Database, idx: u32) -> Result<(), Box<dyn std::error::Error>> {
    let mut streak = get_one(db, idx).unwrap();
    if let Some(check_in) = streak.last_checkin {
        if check_in == Local::now().date_naive() {
            return Ok(())
        }
    }
    streak.checkin();
    db.update(idx, &streak)?;
    db.save()?;
    Ok(())
}

/// Remove a streak
fn delete(db: &mut Database, idx: u32) -> Result<(), Box<dyn std::error::Error>> {
    db.delete(idx)?;
    db.save()?;
    Ok(())
}

/// Builds table of streaks from list
fn build_table(streaks: Vec<Streak>) -> String {
    let mut builder = Builder::new();
    let header_style = Style::new().italic();
    builder.push_record([
        header_style.paint("Streak").to_string(),
        header_style.paint("Freq").to_string(),
        header_style.paint("Status").to_string(),
        header_style.paint("Last Check In").to_string(),
        header_style.paint("Total").to_string()
    ]);

    for streak in streaks.iter() {
        let mut wrapped_text = String::new();
        let wrapped_lines = textwrap::wrap(&streak.task.as_str(), 60);
        let num_lines: u16 = wrapped_lines.len().try_into().unwrap();
        for line in wrapped_lines {
            wrapped_text.push_str(&format!("{}\n", line));
        }

        let streak_name = Style::new().bold().paint(wrapped_text);
        let frequency = Style::new().paint(format!("{}", &streak.frequency));
        let check_in = match &streak.last_checkin {
            Some(date) => date.to_string(),
            None => "None".to_string()
        };
        let last_checkin = Style::new().bold().paint(format!("{:^13}", check_in));
        let total_checkins = Style::new().bold().paint(format!("{:^5}", &streak.total_checkins));
        builder.push_record([
            streak_name.to_string(),
            frequency.to_string(),
            streak.emoji_status(),
            last_checkin.to_string(),
            total_checkins.to_string()
        ]);
    }

    builder
        .index()
        .build()
        .with(TabledStyle::psql())
        .to_string()
}

pub fn get_database_url() -> String {
    let cli = Cli::parse();
    let mut db_url: String = "skidmarks.ron".to_string();
    if cli.database_url != "" {
        db_url = cli.database_url.to_string();
    }
    // Feels hacky, fix with `clio`?
    let db_url = match db_url.chars().nth(0) {
        Some('/') => db_url,
        Some('~') => db_url,
        _ => format!("{}/{db_url}", dirs::data_local_dir().unwrap().display())
    };
    db_url
}

/// Parses command line options
pub fn parse() {
    let cli = Cli::parse();
    let db_url = get_database_url();
    let mut db = Database::new(db_url.as_str()).expect("Could not load database");
    let response_style = Style::new().bold().fg(Color::Green);
    match &cli.command {
        Commands::Add { name, frequency } => match frequency {
            Frequency::Daily => {
                let streak = new_daily(name.to_string(), &mut db).unwrap();
                let response = response_style.paint("Created a new daily streak:").to_string();
                let tada = Emoji("🎉", "");
                println!("{tada} {response} {}", streak.task);
            }
            Frequency::Weekly => {
                let streak = new_weekly(name.to_string(), &mut db).unwrap();
                let response = response_style.paint("Created a new weekly streak:").to_string();
                let tada = Emoji("🎉", "");
                println!("{tada} {response} {}", streak.task);
            }
        },
        Commands::List => {
            let streak_list = get_all(db);
            println!("{}", build_table(streak_list));
        }
        Commands::Get { idx } => {
            let streak = get_one(&mut db, *idx);
            println!("{}", build_table(vec![streak.unwrap()]));
        }
        Commands::CheckIn { idx } => match checkin(&mut db, *idx) {
            Ok(_) => {
                let streak = get_one(&mut db, *idx);
                let name = &streak.unwrap().task;
                let response = response_style.paint(format!("Checked in on the {name} streak!")).to_string();
                let star = Emoji("🌟", "");
                println!("{star} {response}")
            }
            Err(e) => {
                let response = Style::new().bold().fg(Color::Red).paint("Error checking in:");
                eprintln!("{response} {}", e)
            },
        },
        Commands::Remove { idx } => {
            let streak = get_one(&mut db, *idx).clone();
            let _ = delete(&mut db, *idx);
            let name = &streak.unwrap().task;
            let response = response_style.paint(format!(r#"Removed the "{name}" streak"#)).to_string();
            let trash = Emoji("🗑️", "");
            println!("{trash} {response}")
        }
        Commands::Tui => {
            tui::main().expect("Couldn't launch TUI")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use assert_cmd::Command;
    use assert_fs::TempDir;

    lazy_static::lazy_static! {
        static ref FILE_LOCK: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn test_get_all() {
        let temp = TempDir::new().unwrap();

        let mut cmd = Command::cargo_bin("skidmarks").unwrap();
        let list_assert = cmd
            .arg("--database-url")
            .arg(format!("{}{}", temp.path().display(), "test-get-all.ron"))
            .arg("list")
            .assert();
        list_assert.success();
    }

    #[test]
    fn test_new_daily_command() {
        let temp = TempDir::new().unwrap();
        let mut cmd = Command::cargo_bin("skidmarks").unwrap();
        let add_assert = cmd
            .arg("--database-url")
            .arg(format!("{}{}", temp.path().display(), "test-new-daily.ron"))
            .arg("add")
            .arg("--name")
            .arg("Test Streak")
            .arg("--frequency")
            .arg("daily")
            .assert();
        add_assert.success();
    }

    #[test]
    fn test_new_weekly_command() {
        let temp = TempDir::new().unwrap();
        let mut cmd = Command::cargo_bin("skidmarks").unwrap();
        let add_assert = cmd
            .arg("--database-url")
            .arg(format!("{}{}", temp.path().display(), "test-new-weekly.ron"))
            .arg("add")
            .arg("--name")
            .arg("Test Streak")
            .arg("--frequency")
            .arg("weekly")
            .assert();
        add_assert.success();
    }
}
