use std::fmt::Write;
use ansi_term::Style;
use clap::{Parser, Subcommand};
use console::Emoji;
use tabled::{builder::Builder, settings::{Panel, Style as TabledStyle}};
use crate::{
    db::Database,
    streaks::{Frequency, Streak},
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "List all streaks", long_about = None)]
    List,
    #[command(about = "Create a new streak", long_about = None)]
    Add {
        #[clap(short, long, value_enum)]
        frequency: Frequency,

        #[clap(short, long)]
        name: String,
    },
    #[command(about = "Get a single streak", long_about = None)]
    Get { idx: u32 },
    #[command(about = "Check in to a streak", long_about = None)]
    CheckIn { idx: u32 },
    #[command(about = "Remove a streak", long_about = None)]
    Remove { idx: u32 },
}

fn new_daily(name: String, db: &mut Database) -> Result<Streak, Box<dyn std::error::Error>> {
    let streak = Streak::new_daily(name);
    db.streaks.lock().unwrap().push(streak.clone());
    db.save()?;
    Ok(streak)
}

fn new_weekly(name: String, db: &mut Database) -> Result<Streak, Box<dyn std::error::Error>> {
    let streak = Streak::new_weekly(name);
    db.streaks.lock().unwrap().push(streak.clone());
    db.save()?;
    Ok(streak)
}

fn get_all(db: &mut Database) -> Vec<Streak> {
    let streaks = db.streaks.lock();
    match streaks {
        Ok(streaks) => {
            if streaks.is_empty() {
                Vec::<Streak>::new()
            } else {
                streaks.clone()
            }
        }
        Err(e) => panic!("Error getting streaks: {}", e),
    }
}

fn get_one(db: &mut Database, idx: u32) -> Streak {
    db.streaks
        .lock()
        .unwrap()
        .get(idx as usize)
        .unwrap()
        .clone()
}

fn checkin(db: &mut Database, idx: u32) -> Result<(), Box<dyn std::error::Error>> {
    let mut streak = get_one(db, idx);
    streak.checkin();
    db.update(idx, streak)?;
    db.save()?;
    Ok(())
}

fn delete(db: &mut Database, idx: u32) -> Result<(), Box<dyn std::error::Error>> {
    db.streaks.lock().unwrap().remove(idx as usize);
    db.save()?;
    Ok(())
}

#[allow(dead_code)]
fn list_all(db: &mut Database) -> String {
    let list: Vec<Streak> = get_all(db);
    list.into_iter()
        .enumerate()
        .fold(String::new(), |mut acc, (i, s)| {
            let _ = writeln!(acc, "{}: {}", i + 1, s.task.clone());
            acc
        })
}

pub fn parse(db: &mut Database) {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Add { name, frequency } => match frequency {
            Frequency::Daily => {
                let streak = new_daily(name.to_string(), db).unwrap();
                println!("Created new daily streak: {}", streak.task);
            }
            Frequency::Weekly => {
                let streak = new_weekly(name.to_string(), db).unwrap();
                println!("Created new weekly streak: {}", streak.task);
            }
        },
        Commands::List => {
            let mut builder = Builder::new();
            builder.push_record(["Task", "Freq", "Checked In", "Last Check In"]);

            let streak_list = get_all(db);
            for streak in streak_list.iter() {
                let streak_name = Style::new().bold().paint(&streak.task);
                let frequency = Style::new().italic().paint(format!("{}", &streak.frequency));
                let checked_in = if streak.clone().was_missed() {
                    Emoji("❌", "")
                } else {
                    Emoji("✅", "")
                };
                let last_checkin = Style::new().underline().paint(format!("{}", &streak.last_checkin));
                builder.push_record([
                    streak_name.to_string(),
                    frequency.to_string(),
                    checked_in.to_string(),
                    last_checkin.to_string()
                ]);
            }

            let table = builder
                .index()
                .build()
                .with(TabledStyle::markdown())
                .with(Panel::horizontal(0, "Skidmarks"))
                .to_string();

            println!("{table}");
        }
        Commands::Get { idx } => {
            let streak = get_one(db, *idx - 1);
            println!("{}: {}\n{}", idx, streak.task, streak.frequency);
        }
        Commands::CheckIn { idx } => match checkin(db, *idx - 1) {
            Ok(_) => {
                let streak = get_one(db, *idx - 1);
                println!("Checked in streak: {}", streak.task)
            }
            Err(e) => eprintln!("Error checking in: {}", e),
        },
        Commands::Remove { idx } => {
            let _ = delete(db, *idx - 1);
            println!("Removed streak at index {}", idx)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs};
    use std::sync::Mutex;
    use assert_cmd::Command;
    use crate::settings::Settings;

    lazy_static::lazy_static! {
        static ref FILE_LOCK: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn test_get_all() {
        env::set_var("RUN_MODE", "Testing");
        let settings = Settings::new().unwrap();
        let _lock = FILE_LOCK.lock().unwrap();
        if fs::remove_file(&settings.database.url).is_ok() {
            println!("Removed existing database file");
        } else {
            println!("No existing database file to remove");
        }

        let mut cmd = Command::cargo_bin("skidmarks").unwrap();
        let list_assert = cmd
            .arg("list")
            .assert();
        list_assert.success();

        fs::remove_file(&settings.database.url).unwrap();
    }

    #[test]
    fn test_new_daily_command() {
        env::set_var("RUN_MODE", "Testing");
        let settings = Settings::new().unwrap();
        let _lock = FILE_LOCK.lock().unwrap();
        if fs::remove_file(&settings.database.url).is_ok() {
            println!("Removed existing database file");
        } else {
            println!("No existing database file to remove");
        }

        let mut cmd = Command::cargo_bin("skidmarks").unwrap();
        let add_assert = cmd
            .arg("add")
            .arg("--name")
            .arg("Test Streak")
            .arg("--frequency")
            .arg("daily")
            .assert();
        add_assert.success();

        fs::remove_file(&settings.database.url).unwrap();
    }

    #[test]
    fn test_new_weekly_command() {
        env::set_var("RUN_MODE", "Testing");
        let settings = Settings::new().unwrap();
        let _lock = FILE_LOCK.lock().unwrap();
        if fs::remove_file(&settings.database.url).is_ok() {
            println!("Removed existing database file");
        } else {
            println!("No existing database file to remove");
        }

        let mut cmd = Command::cargo_bin("skidmarks").unwrap();
        let add_assert = cmd
            .arg("add")
            .arg("--name")
            .arg("Test Streak")
            .arg("--frequency")
            .arg("weekly")
            .assert();
        add_assert.success();

        fs::remove_file(&settings.database.url).unwrap();
    }
}
