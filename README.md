# Skidmarks

Skidmarks is a command-line application written in Rust for managing streaks.
It allows users to add and list streaks with different frequencies (daily or
weekly).

## Features

- Add new streaks with a specified name and frequency.
- List all existing streaks.

## Installation

To install Skidmarks, you need to have Rust and Cargo installed on your system.
You can install Rust using [rustup](https://rustup.rs/).

```sh
# Clone the repository
git clone https://github.com/yourusername/skidmarks.git

# Navigate to the project directory
cd skidmarks

# Build the project
cargo build --release
```

## Usage
### Adding a Streak
To add a new streak, use the `add` command with the `--name` and `--frequency`
options.

```sh
# Add a daily streak
./target/release/skidmarks add --name "Exercise" --frequency daily

# Add a weekly streak
./target/release/skidmarks add --name "Read a book" --frequency weekly
```

### Listing All Streaks
To list all existing streaks, use the `list-all` command.

```sh
./target/release/skidmarks list-all
```

## Running Tests
To run the tests for this project, use the following command:

```sh
cargo test
```

## Example Tests
Here are some example tests from the `cli.rs` file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_daily_command() {
        let cli = Cli::parse_from([
            "skidmarks",
            "add",
            "--name",
            "Test Streak",
            "--frequency",
            "daily",
        ]);
        assert!(matches!(cli.command,
            Commands::Add { frequency, name } if frequency == Frequency::Daily && name == "Test Streak"));
    }

    #[test]
    fn test_new_weekly_command() {
        let cli = Cli::parse_from([
            "skidmarks",
            "add",
            "--name",
            "Test Streak",
            "--frequency",
            "weekly",
        ]);
        assert!(matches!(cli.command,
            Commands::Add { frequency, name }
            if frequency == Frequency::Weekly
            && name == "Test Streak"
        ));
    }
}
```

## Example
Here is an example of how to use Skidmarks:

```sh
# Add a daily streak
./target/release/skidmarks add --name "Test Streak" --frequency daily

# Add a weekly streak
./target/release/skidmarks add --name "Test Streak" --frequency weekly

# List all streaks
./target/release/skidmarks list-all
```

## Contributing
Contributions are welcome! Please open an issue or submit a pull request for
any improvements or bug fixes.

## License
This project is licensed under the Apache License. See the [LICENSE](LICENSE)
file for details.
