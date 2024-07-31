# Skidmarks

Skidmarks is a command-line application written in Rust for managing streaks.
It allows users to add and list streaks with different frequencies (daily or
weekly).

## Features

- Add new streaks with a specified name and frequency.
- List all existing streaks.
- Check in on a streak to keep it going.
- Remove a streak when it's no longer needed.

## Installation

To install Skidmarks, you need to have Rust and Cargo installed on your system.
You can install Rust using [rustup](https://rustup.rs/).

```sh
cargo install --locked skidmarks
```

Or manually via:

```sh
# Clone the repository
git clone https://github.com/kennethlove/skidmarks.git

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
$ skidmarks add --name "Exercise" --frequency daily

🎉 Created a new daily streak: Exercise
```

```sh
# Add a weekly streak
$ skidmarks add --name "Read a book" --frequency weekly

🎉 Created a new weekly streak: Read a book
```

### Listing All Streaks

To list all existing streaks, use the `list` command.

```sh
$ skidmarks list

   | Streak          | Freq   | Status | Last Check In | Total
---+-----------------+--------+--------+---------------+-------
 0 | Exercise        | daily  | ✅     |  2024-07-31   |   1
 1 | Wordle          | daily  | ✅     |  2024-07-31   |   1
 2 | Coloring page   | daily  | ✅     |  2024-07-31   |   1
 3 | Duolingo        | daily  | ✅     |  2024-07-31   |   1
 4 | Animal Crossing | daily  | ❌     |     None      |   0
 5 | Read a book     | weekly | ❌     |     None      |   0
```

### Checking In on a Streak

To check in on a streak, use the `check-in <streak id>` command.

```sh
$ skidmarks check-in 0

🌟 Checked in on the "Exercise" streak!
```

### Removing a Streak

To remove a streak, use the `remove <streak id>` command.

```sh
$ skidmarks remove 5

🗑 Removed the "Read a book" streak
```

## Running Tests

To run the tests for this project, use the following command:

```sh
cargo test
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for
any improvements or bug fixes.

## License

This project is licensed under the Apache License. See the [LICENSE](LICENSE)
file for details.
