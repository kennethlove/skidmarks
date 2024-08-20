use crate::color::AppStyles;
use crate::streak::Streak;
use ansi_term::Style;
use tabled::{builder::Builder, settings::Style as TabledStyle};
use term_size::dimensions;

/// Builds table of streaks from list
pub fn build_table(streaks: Vec<Streak>) -> String {
    let app_styles = AppStyles::new();
    let mut builder = Builder::new();
    let header_style = Style::new().italic().fg(app_styles.table_header_fg);
    builder.push_record([
        header_style.paint("\nIdent").to_string(),
        header_style.paint("\nTask").to_string(),
        header_style.paint("\nFreq").to_string(),
        header_style.paint("\nStatus").to_string(),
        header_style.paint("\nLast Check In").to_string(),
        header_style.paint("Current\nStreak").to_string(),
        header_style.paint("Longest\nStreak").to_string(),
        header_style.paint("\nTotal").to_string(),
    ]);

    let (width, _) = match dimensions() {
        Some((w, _)) => (w, 0),
        None => (60, 0),
    };
    let width = std::cmp::min(width.saturating_sub(60), 30);

    for streak in streaks.iter() {
        let mut wrapped_text = String::new();
        let wrapped_lines = textwrap::wrap(&streak.task.as_str(), width);
        for line in wrapped_lines {
            wrapped_text.push_str(&format!("{line}\n"));
        }
        wrapped_text = wrapped_text.trim().to_string();

        let id = &streak.id.to_string()[0..5];
        let index = Style::new().bold().paint(format!("{}", id));
        let streak_name = Style::new().bold().paint(wrapped_text);
        let frequency = Style::new().paint(format!("{:^6}", &streak.frequency));
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
