use std::path::Path;

use ansi_term::{Color, Style};
use catppuccin::PALETTE;
use clap::{Parser, Subcommand};
use console::Emoji;
use dirs;
use uuid::Uuid;

use crate::{
    cli::table::build_table,
    db::Database,
    gui,
    sorting::get_sort_order,
    streak::{sort_streaks, Frequency, Streak},
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
    List {
        #[arg(long, default_value = "task+", help = "Sort by field")]
        sort_by: String,

        #[arg(long, default_value = "", help = "Search for task")]
        search: String,

        #[arg(long, action, group = "frequency", help = "Show daily streaks")]
        daily: bool,

        #[arg(long, action, group = "frequency", help = "Show weekly streaks")]
        weekly: bool,

        #[arg(long, action, group = "status", help = "Show done streaks")]
        done: bool,

        #[arg(long, action, group = "status", help = "Show waiting streaks")]
        waiting: bool,

        #[arg(long, action, group = "status", help = "Show missed streaks")]
        missed: bool,
    },
    #[command(about = "Create a new streak", long_about = None, short_flag = 'a')]
    Add {
        #[clap(short, long, value_enum)]
        frequency: Frequency,

        #[clap(short, long)]
        task: String,
    },
    #[command(about = "Get one streak", long_about = None, short_flag='o')]
    Get { ident: String },
    #[command(about = "Check in to a streak", long_about = None, short_flag = 'c')]
    CheckIn { ident: String },
    #[command(about = "Remove a streak", long_about = None, short_flag = 'r')]
    Remove { ident: String },
    #[command(about = "Switch to TUI", long_about = None)]
    Tui,
    #[command(about = "Switch to GUI", long_about = None)]
    Gui,
}

/// Create a new daily streak item
fn new_daily(task: String, db: &mut Database) -> Result<Streak, Box<dyn std::error::Error>> {
    let streak = Streak::new_daily(task);
    db.streaks.push(streak.clone());
    db.save()?;
    Ok(streak)
}

/// Create a new weekly streak item
fn new_weekly(task: String, db: &mut Database) -> Result<Streak, Box<dyn std::error::Error>> {
    let streak = Streak::new_weekly(task);
    db.streaks.push(streak.clone());
    db.save()?;
    Ok(streak)
}

#[allow(dead_code)]
/// Get all streaks
fn get_all(mut db: Database) -> Vec<Streak> {
    db.get_all()
}

/// Get one single streak
#[allow(dead_code)]
fn get_one(db: &mut Database, id: Uuid) -> Option<Streak> {
    let streak = db.streaks.clone().into_iter().find(|s| s.id == id);
    if let Some(streak) = streak {
        return Some(streak.clone());
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
fn checkin(db: &mut Database, ident: &str) -> Result<(), Box<dyn std::error::Error>> {
    let streak = db.get_by_id(ident).unwrap();
    match db.checkin(streak.id) {
        Ok(_) => {
            db.save()?;
            Ok(())
        }
        Err(e) => Err(Box::new(e)),
    }
}

/// Remove a streak
fn delete(db: &mut Database, ident: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(streak) = get_one_by_id(db, ident) {
        db.delete(streak.id)?;
        db.save()?;
    }
    Ok(())
}

pub fn get_database_url() -> String {
    let cli = Cli::parse();
    let path = Path::new(&dirs::data_local_dir().unwrap()).join(cli.database_url);
    path.to_string_lossy().to_string()
}

const fn ansi(color: &catppuccin::Color) -> ansi_term::Colour {
    ansi_term::Colour::RGB(color.rgb.r, color.rgb.g, color.rgb.b)
}

/// Parses command line options
pub fn parse() {
    let cli = Cli::parse();
    let db_url = get_database_url();
    let mut db = Database::new(&db_url).expect("Could not load database");
    let response_style = Style::new().bold().fg(ansi(&PALETTE.mocha.colors.mauve));
    match &cli.command {
        Commands::Add { task, frequency } => match frequency {
            Frequency::Daily => {
                let streak = new_daily(task.to_string(), &mut db).unwrap();
                let response = response_style
                    .paint("Created a new daily streak:")
                    .to_string();
                let tada = Emoji("🎉", "");
                println!("{tada} {response} {}", streak.task);
            }
            Frequency::Weekly => {
                let streak = new_weekly(task.to_string(), &mut db).unwrap();
                let response = response_style
                    .paint("Created a new weekly streak:")
                    .to_string();
                let tada = Emoji("🎉", "");
                println!("{tada} {response} {}", streak.task);
            }
        },
        Commands::List {
            sort_by,
            search,
            daily,
            weekly,
            done,
            waiting,
            missed,
        } => {
            let mut streak_list = match search.is_empty() {
                true => db.get_all(),
                false => db.search(search),
            };
            let sort_by = get_sort_order(sort_by);

            if *daily {
                streak_list = streak_list
                    .into_iter()
                    .filter(|s| s.frequency == Frequency::Daily)
                    .collect();
            }

            if *weekly {
                streak_list = streak_list
                    .into_iter()
                    .filter(|s| s.frequency == Frequency::Weekly)
                    .collect();
            }

            if *done {
                streak_list = streak_list.into_iter().filter(|s| s.is_done()).collect();
            }

            if *missed {
                streak_list = streak_list.into_iter().filter(|s| s.is_missed()).collect();
            }

            if *waiting {
                streak_list = streak_list.into_iter().filter(|s| s.is_waiting()).collect();
            }

            streak_list = sort_streaks(streak_list, sort_by.0, sort_by.1);
            println!("{}", build_table(streak_list));
        }
        Commands::Get { ident } => {
            let streak = vec![db.get_by_id(&ident).unwrap()];
            println!("{}", build_table(streak));
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
        Commands::Remove { ident } => {
            let streak = db.get_by_id(&ident).unwrap();
            let _ = delete(&mut db, &ident);
            let name = &streak.task;
            let response = response_style.paint("Removed:").to_string();
            let trash = Emoji("🗑️", "");
            println!("{trash} {response} {}", name);
        }
        Commands::Tui => tui::main().expect("Couldn't launch TUI"),
        Commands::Gui => gui::main(),
    }
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use assert_fs::TempDir;
    use rstest::*;

    #[fixture]
    pub fn command() -> Command {
        Command::cargo_bin("skidmarks").unwrap()
    }

    #[rstest]
    fn get_all(mut command: Command) {
        let temp = TempDir::new().unwrap();

        command
            .arg("--database-url")
            .arg(format!("{}/{}", temp.path().display(), "test-get-all.ron"))
            .arg("list")
            .assert()
            .success();
    }

    #[rstest]
    fn new_daily_command(mut command: Command) {
        let temp = TempDir::new().unwrap();
        command
            .arg("--database-url")
            .arg(format!(
                "{}/{}",
                temp.path().display(),
                "test-new-daily.ron"
            ))
            .arg("add")
            .arg("--task")
            .arg("Test Streak")
            .arg("--frequency")
            .arg("daily")
            .assert()
            .success();
    }

    #[rstest]
    fn new_weekly_command(mut command: Command) {
        let temp = TempDir::new().unwrap();
        command
            .arg("--database-url")
            .arg(format!(
                "{}/{}",
                temp.path().display(),
                "test-new-weekly.ron"
            ))
            .arg("add")
            .arg("--task")
            .arg("Test Streak")
            .arg("--frequency")
            .arg("weekly")
            .assert()
            .success();
    }

    #[rstest]
    fn test_sort_order(
        #[values(
            "task+",
            "task-",
            "frequency+",
            "frequency-",
            "last_checkin+",
            "last_checkin-",
            "current_streak+",
            "current_streak-",
            "longest_streak+",
            "longest_streak-",
            "total_checkins+",
            "total_checkins-"
        )]
        sort_string: &str,
        mut command: Command,
    ) {
        let temp = TempDir::new().unwrap();
        let mut list_assert = command
            .arg("--database-url")
            .arg(format!(
                "{}/{}",
                temp.path().display(),
                "test-sort-order.ron"
            ))
            .arg("list")
            .arg("--sort-by");

        #[cfg(target_os = "windows")]
        {
            list_assert = list_assert.arg(format!(r#""{}""#, sort_string));
        }
        #[cfg(not(target_os = "windows"))]
        {
            list_assert = list_assert.arg(format!("{}", sort_string));
        }
        list_assert.assert().success();
    }

    #[rstest]
    fn test_search(mut command: Command) {
        let temp = TempDir::new().unwrap();

        command
            .arg("--database-url")
            .arg(format!("{}/{}", temp.path().display(), "test-search.ron"))
            .arg("list")
            .arg("--search")
            .arg("Test")
            .assert()
            .success();
    }

    #[rstest]
    fn test_search_and_sort(mut command: Command) {
        let temp = TempDir::new().unwrap();

        command
            .arg("--database-url")
            .arg(format!(
                "{}/{}",
                temp.path().display(),
                "test-search-sort.ron"
            ))
            .arg("list")
            .arg("--search")
            .arg("Test")
            .arg("--sort-by")
            .arg("task+")
            .assert()
            .success();
    }

    #[rstest]
    fn test_frequency_filter(mut command: Command) {
        let temp = TempDir::new().unwrap();

        command
            .arg("--database-url")
            .arg(format!(
                "{}/{}",
                temp.path().display(),
                "test-frequency-filter.ron"
            ))
            .arg("list")
            .arg("--frequency")
            .arg("daily")
            .assert()
            .success();
    }

    #[rstest]
    fn test_frequency_filter_and_sort(mut command: Command) {
        let temp = TempDir::new().unwrap();

        command
            .arg("--database-url")
            .arg(format!(
                "{}/{}",
                temp.path().display(),
                "test-frequency-filter-sort.ron"
            ))
            .arg("list")
            .arg("--frequency")
            .arg("daily")
            .arg("--sort-by")
            .arg("task+")
            .assert()
            .success();
    }

    #[rstest]
    #[case("done")]
    #[case("missed")]
    #[case("waiting")]
    fn test_filter_by_status(mut command: Command, #[case] status: &str) {
        let temp = TempDir::new().unwrap();

        command
            .arg("--database-url")
            .arg(format!(
                "{}/{}",
                temp.path().display(),
                "test-filter-by-status.ron"
            ))
            .arg("list")
            .arg(format!("--{status}"))
            .assert()
            .success();
    }
}
