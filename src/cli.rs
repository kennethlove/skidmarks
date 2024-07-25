use crate::streaks::{Frequency, Streak};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Create a new streak", long_about = None)]
    Add {
        #[clap(short, long, value_enum)]
        frequency: Frequency,

        #[clap(short, long)]
        name: String,
    },
}

fn new_daily(name: String) -> String {
    let streak = Streak::new_daily(name);
    format!("Created {} streak: {}", streak.frequency, streak.task)
}

fn new_weekly(name: String) -> String {
    let streak = Streak::new_weekly(name);
    format!("Created {} streak: {}", streak.frequency, streak.task)
}

pub fn parse() -> String {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Add { name, frequency } => match frequency {
            Frequency::Daily => new_daily(name.to_string()),
            Frequency::Weekly => new_weekly(name.to_string()),
        },
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
