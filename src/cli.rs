use ansi_term::Style;
use clap::{Parser, Subcommand};
use console::Emoji;
use tabled::{builder::Builder, settings::{Panel, Style as TabledStyle}};
use crate::{
    db::Database,
    settings::Settings,
    streaks::{Frequency, Streak, Status},
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

fn build_table(streaks: Vec<Streak>) -> String {
    let mut builder = Builder::new();
    builder.push_record(["Streak", "Freq", "Status", "Last Check In", "Total"]);

    for streak in streaks.iter() {
        let streak_name = Style::new().bold().paint(&streak.task);
        let frequency = Style::new().italic().paint(format!("{}", &streak.frequency));
        let checked_in = match streak.clone().status() {
            Status::Done => Emoji("✅", ""),
            Status::Missed => Emoji("❌", ""),
            Status::Waiting => Emoji("⏳", ""),
        };
        let check_in = match &streak.last_checkin {
            Some(date) => date.to_string(),
            None => "None".to_string()
        };
        let last_checkin = Style::new().underline().paint(format!("{}", check_in));
        let total_checkins = Style::new().bold().paint(format!("{}", &streak.total_checkins));
        builder.push_record([
            streak_name.to_string(),
            frequency.to_string(),
            checked_in.to_string(),
            last_checkin.to_string(),
            total_checkins.to_string()
        ]);
    }

    builder
        .index()
        .build()
        .with(TabledStyle::psql())
        .with(Panel::header("Skidmarks"))
        .to_string()
}

pub fn parse() {
    let cli = Cli::parse();
    let settings = Settings::new().unwrap();
    let db_url: String;
    if cli.database_url != settings.database.url {
        db_url = cli.database_url.clone();
    } else {
        db_url = settings.database.url;
    }
    let mut db = Database::new(db_url.as_str()).expect("Could not load database");
    match &cli.command {
        Commands::Add { name, frequency } => match frequency {
            Frequency::Daily => {
                let streak = new_daily(name.to_string(), &mut db).unwrap();
                println!("Created new daily streak: {}", streak.task);
            }
            Frequency::Weekly => {
                let streak = new_weekly(name.to_string(), &mut db).unwrap();
                println!("Created new weekly streak: {}", streak.task);
            }
        },
        Commands::List => {
            let streak_list = get_all(&mut db);
            println!("{}", build_table(streak_list));
        }
        Commands::Get { idx } => {
            let streak = get_one(&mut db, *idx);
            println!("{}", build_table(vec![streak]));
        }
        Commands::CheckIn { idx } => match checkin(&mut db, *idx) {
            Ok(_) => {
                let streak = get_one(&mut db, *idx);
                println!("Checked in streak: {}", streak.task)
            }
            Err(e) => eprintln!("Error checking in: {}", e),
        },
        Commands::Remove { idx } => {
            let _ = delete(&mut db, *idx);
            println!("Removed streak at index {}", idx)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::sync::Mutex;
    use assert_cmd::Command;

    lazy_static::lazy_static! {
        static ref FILE_LOCK: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn test_get_all() {
        env::set_var("RUN_MODE", "Testing");
        let mut cmd = Command::cargo_bin("skidmarks").unwrap();
        let list_assert = cmd
            .arg("--database-url")
            .arg("test-get-all.ron")
            .arg("list")
            .assert();
        list_assert.success();
    }

    #[test]
    fn test_new_daily_command() {
        env::set_var("RUN_MODE", "Testing");
        let mut cmd = Command::cargo_bin("skidmarks").unwrap();
        let add_assert = cmd
            .arg("--database-url")
            .arg("test-new-daily.ron")
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
        env::set_var("RUN_MODE", "Testing");
        let mut cmd = Command::cargo_bin("skidmarks").unwrap();
        let add_assert = cmd
            .arg("--database-url")
            .arg("test-new-weekly.ron")
            .arg("add")
            .arg("--name")
            .arg("Test Streak")
            .arg("--frequency")
            .arg("weekly")
            .assert();
        add_assert.success();
    }
}
