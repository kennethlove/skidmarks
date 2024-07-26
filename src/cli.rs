use crate::{
    db::Database,
    streaks::{Frequency, Streak},
};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    ListAll,
    #[command(about = "Create a new streak", long_about = None)]
    Add {
        #[clap(short, long, value_enum)]
        frequency: Frequency,

        #[clap(short, long)]
        name: String,
    },
    Get {
        idx: u32,
    },
    CheckIn {
        idx: u32,
    },
    Remove {
        idx: u32,
    },
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
    db.streaks.lock().unwrap().clone()
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
        Commands::ListAll => {
            let list: Vec<Streak> = get_all(db);
            let output: String = list
                .into_iter()
                .enumerate()
                .map(|(i, s)| format!("{}: {}\n", i + 1, s.task.clone()))
                .collect();

            println!("{}", output);
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
    use super::*;

    #[test]
    fn test_new_daily_command() {
        let cli = Cli::parse_from(&[
            "skidmarks",
            "add",
            "--name",
            "Test Streak",
            "--frequency",
            "daily",
        ]);
        assert!(matches!(cli.command, Commands::Add { .. }));
    }
}
