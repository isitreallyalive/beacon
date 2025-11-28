use std::cell::LazyCell;
use std::env;

use log::Level;
use ratatui::prelude::*;
use tui_logger::{ExtLogRecord, LogFormatter};

const DEFAULT_TARGET: &str = "beacon";
const MAX_TARGET_LEN: LazyCell<usize> = LazyCell::new(|| {
    env::var("MAX_TARGET_LEN")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_TARGET.len())
});

pub struct Formatter;

fn get_target(evt: &ExtLogRecord) -> String {
    let target = evt
        .target()
        .split("::")
        .next()
        .map(|s| s.trim_start_matches("beacon_"))
        .unwrap_or(DEFAULT_TARGET);
    let len = target.chars().count();
    if len >= *MAX_TARGET_LEN {
        target.to_string()
    } else {
        format!("{:width$}", target, width = *MAX_TARGET_LEN)
    }
}

impl LogFormatter for Formatter {
    fn min_width(&self) -> u16 {
        4
    }

    fn format(&self, _: usize, evt: &ExtLogRecord) -> Vec<Line<'_>> {
        let level = match evt.level {
            Level::Info => Span::styled("INFO", Style::default().fg(Color::Green).bold()),
            Level::Warn => Span::styled("WARN", Style::default().fg(Color::Yellow).bold()),
            Level::Error => Span::styled("ERROR", Style::default().fg(Color::Red).bold()),
            Level::Debug => Span::styled("DEBUG", Style::default().fg(Color::Blue).bold()),
            Level::Trace => Span::styled("TRACE", Style::default().fg(Color::DarkGray).bold()),
        };
        let timestamp = Span::styled(
            evt.timestamp.format("%H:%M:%S").to_string(),
            Style::default().fg(Color::DarkGray),
        );
        let message = Span::raw(evt.msg().to_string());

        vec![Line::from(vec![
            level,
            " ".into(),
            timestamp,
            "  ".into(),
            get_target(evt).gray().into(),
            "  ".into(),
            message,
        ])]
    }
}
