use std::collections::HashMap;
use std::path::Path;

use ansi_term::{Color, Style};
#[allow(unused_imports)]
use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use console::Emoji;
use dirs;
use tabled::{builder::Builder, settings::Style as TabledStyle};
use uuid::Uuid;

use crate::{
    db::Database,
    streak::{Frequency, Streak},
    tui,
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[clap(short, long, default_value = "skidmarks.ron")]
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
    #[command(about = "Get one streak", long_about = None, short_flag='o')]
    Get { ident: String },
    #[command(about = "Check in to a streak", long_about = None, short_flag = 'c')]
    CheckIn { ident: String },
    #[command(about = "Remove a streak", long_about = None, short_flag = 'r')]
    Remove { idx: u32 },
    #[command(about = "Switch to TUI", long_about = None, short_flag = 't')]
    Tui,
}

/// Create a new daily streak item
fn new_daily(name: String, db: &mut Database) -> Result<Streak, Box<dyn std::error::Error>> {
    let streak = Streak::new_daily(name);
    db.streaks.lock().unwrap().insert(streak.id, streak.clone());
    db.save()?;
    Ok(streak)
}

/// Create a new weekly streak item
fn new_weekly(name: String, db: &mut Database) -> Result<Streak, Box<dyn std::error::Error>> {
    let streak = Streak::new_weekly(name);
    db.streaks.lock().unwrap().insert(streak.id, streak.clone());
    db.save()?;
    Ok(streak)
}

/// Get all streaks
fn get_all(mut db: Database) -> HashMap<Uuid, Streak> {
    match db.get_all() {
        Some(streaks) => streaks.clone(),
        None => HashMap::<Uuid, Streak>::new(),
    }
}

/// Get one single streak
#[allow(dead_code)]
fn get_one(db: &mut Database, id: Uuid) -> Option<Streak> {
    if let Some(streak) = db.streaks.lock().unwrap().get(&id) {
        return Some(streak.clone());
    }
    None
}

fn get_one_by_index(db: &mut Database, idx: usize) -> Option<Streak> {
    if let Some(streak) = db.get_by_index(idx) {
        return Some(streak);
    }
    None
}

fn get_one_by_id(db: &mut Database, ident: &str) -> Option<Streak> {
    if let Some(streak) = db.get_by_id(ident) {
        return Some(streak);
    }
    None
}

/// Check in to a streak today
/// Index is decremented by 1 to match 0-based indexing
fn checkin(db: &mut Database, ident: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut streak = get_one_by_id(db, ident).unwrap();
    if let Some(check_in) = streak.last_checkin {
        if check_in == Local::now().date_naive() {
            return Ok(());
        }
    }
    streak.checkin();
    db.update(streak.id, streak)?;
    db.save()?;
    Ok(())
}

/// Remove a streak
fn delete(db: &mut Database, id: usize) -> Result<(), Box<dyn std::error::Error>> {
    let id = get_one_by_index(db, id).unwrap().id;
    db.delete(id)?;
    db.save()?;
    Ok(())
}

/// Builds table of streaks from list
fn build_table(streaks: HashMap<Uuid, Streak>) -> String {
    let mut builder = Builder::new();
    let header_style = Style::new().italic();
    builder.push_record([
        header_style.paint("\nIdent").to_string(),
        header_style.paint("\nStreak").to_string(),
        header_style.paint("\nFreq").to_string(),
        header_style.paint("\nStatus").to_string(),
        header_style.paint("\nLast Check In").to_string(),
        header_style.paint("Current\nStreak").to_string(),
        header_style.paint("Longest\nStreak").to_string(),
        header_style.paint("\nTotal").to_string(),
    ]);
    // let zipper: Vec<_> = (1..).zip(streaks.iter()).collect();
    // for (idx, (_id, streak)) in zipper.iter() {
    for (id, streak) in streaks.iter() {
        let mut wrapped_text = String::new();
        let wrapped_lines = textwrap::wrap(&streak.task.as_str(), 60);
        for line in wrapped_lines {
            wrapped_text.push_str(&format!("{line}"));
        }

        let id = &id.to_string()[0..5];
        let index = Style::new().bold().paint(format!("{}", id));
        let streak_name = Style::new().bold().paint(wrapped_text);
        let frequency = Style::new().paint(format!("{}", &streak.frequency));
        let emoji = Style::new().paint(format!("{:^6}", &streak.emoji_status()));
        let check_in = match &streak.last_checkin {
            Some(date) => date.to_string(),
            None => "None".to_string(),
        };
        let last_checkin = Style::new().bold().paint(format!("{:^13}", check_in));
        let current_streak = Style::new()
            .bold()
            .paint(format!("{:^7}", &streak.current_streak));
        let longest_streak = Style::new()
            .bold()
            .paint(format!("{:^7}", &streak.longest_streak));
        let total_checkins = Style::new()
            .bold()
            .paint(format!("{:^5}", &streak.total_checkins));
        builder.push_record([
            index.to_string(),
            streak_name.to_string(),
            frequency.to_string(),
            emoji.to_string(),
            last_checkin.to_string(),
            current_streak.to_string(),
            longest_streak.to_string(),
            total_checkins.to_string(),
        ]);
    }

    builder.build().with(TabledStyle::psql()).to_string()
}

pub fn get_database_url() -> String {
    let cli = Cli::parse();
    let path = Path::new(&dirs::data_local_dir().unwrap()).join(cli.database_url);
    path.to_string_lossy().to_string()
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
                let response = response_style
                    .paint("Created a new daily streak:")
                    .to_string();
                let tada = Emoji("🎉", "");
                println!("{tada} {response} {}", streak.task);
            }
            Frequency::Weekly => {
                let streak = new_weekly(name.to_string(), &mut db).unwrap();
                let response = response_style
                    .paint("Created a new weekly streak:")
                    .to_string();
                let tada = Emoji("🎉", "");
                println!("{tada} {response} {}", streak.task);
            }
        },
        Commands::List => {
            let streak_list = get_all(db);
            println!("{}", build_table(streak_list));
        }
        Commands::Get { ident } => {
            let streak = db.get_by_id(&ident).unwrap();
            let mut hash = HashMap::<Uuid, Streak>::new();
            hash.insert(streak.id, streak);
            println!("{}", build_table(hash));
        }
        Commands::CheckIn { ident } => match checkin(&mut db, ident) {
            Ok(_) => {
                let streak = db.get_by_id(&ident).unwrap();
                let response = response_style.paint("Checked in on").to_string();
                let star = Emoji("🌟", "");
                println!("{star} {response}: {}", streak.task);
            }
            Err(e) => {
                let response = Style::new()
                    .bold()
                    .fg(Color::Red)
                    .paint("Error checking in:");
                eprintln!("{response} {}", e)
            }
        },
        Commands::Remove { idx } => {
            let streak = db.get_by_index(*idx as usize).unwrap();
            let _ = delete(&mut db, *idx as usize);
            let name = &streak.task;
            let response = response_style.paint("Removed:").to_string();
            let trash = Emoji("🗑️", "");
            println!("{trash} {response} {}", name);
        }
        Commands::Tui => tui::main().expect("Couldn't launch TUI"),
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
    fn get_all() {
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
    fn new_daily_command() {
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
    fn new_weekly_command() {
        let temp = TempDir::new().unwrap();
        let mut cmd = Command::cargo_bin("skidmarks").unwrap();
        let add_assert = cmd
            .arg("--database-url")
            .arg(format!(
                "{}{}",
                temp.path().display(),
                "test-new-weekly.ron"
            ))
            .arg("add")
            .arg("--name")
            .arg("Test Streak")
            .arg("--frequency")
            .arg("weekly")
            .assert();
        add_assert.success();
    }
}
