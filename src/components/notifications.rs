use std::collections::VecDeque;

use ratatui::{
    prelude::*,
    style::Color,
    symbols::border,
    widgets::{Block, Paragraph, StatefulWidget, Wrap},
};
use throbber_widgets_tui::{Throbber, ThrobberState};

/// Severity of a notification. The variants are declared from least to most
/// severe so the derived `Ord` can pick the worst pending notification for a
/// screen; do not reorder them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Info,
    Warning,
    Error,
    Critical,
}

impl Level {
    pub fn color(&self) -> Color {
        match self {
            Level::Info => Color::Cyan,
            Level::Warning => Color::Yellow,
            Level::Error => Color::Red,
            Level::Critical => Color::Red,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Level::Info => "INFO",
            Level::Warning => "WAARSCHUWING",
            Level::Error => "FOUT",
            Level::Critical => "KRITIEK",
        }
    }

    /// Short bracketed marker shown in front of menu items and in the
    /// notification title. Errors and warnings share `[!]` and differ only in
    /// color, so the marker alone is readable on a monochrome terminal while
    /// the color carries the severity on a color one.
    pub fn marker(&self) -> &'static str {
        match self {
            Level::Info => "[?]",
            Level::Warning | Level::Error | Level::Critical => "[!]",
        }
    }

    /// The marker styled in the level's color, ready to be dropped into a line.
    pub fn marker_span(&self) -> Span<'static> {
        self.marker().bold().fg(self.color())
    }
}

/// A single queued message. `target` names the screen the message is about,
/// if any, so the menu can flag that screen with the level's marker.
#[derive(Debug, Clone)]
pub struct Notification<T> {
    pub level: Level,
    pub message: String,
    pub target: Option<T>,
}

/// FIFO of notifications, generic over the type that identifies a screen so
/// this component does not depend on the app's menu enum.
#[derive(Debug)]
pub struct Notifications<T> {
    queue: VecDeque<Notification<T>>,
}

impl<T> Default for Notifications<T> {
    fn default() -> Self {
        Self { queue: VecDeque::new() }
    }
}

impl<T: PartialEq> Notifications<T> {
    pub fn push(&mut self, level: Level, message: impl Into<String>, target: Option<T>) {
        self.queue.push_back(Notification {
            level,
            message: message.into(),
            target,
        });
    }

    pub fn dismiss(&mut self) {
        self.queue.pop_front();
    }

    /// The most severe level among all pending notifications aimed at
    /// `target`, not just the one currently displayed, so a screen stays
    /// flagged until every message about it has been dismissed.
    pub fn highest_level_for(&self, target: &T) -> Option<Level> {
        self.queue
            .iter()
            .filter(|n| n.target.as_ref() == Some(target))
            .map(|n| n.level)
            .max()
    }
}

impl<T> StatefulWidget for &Notifications<T> {
    type State = ThrobberState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let Some(current) = self.queue.front() else {
            return;
        };

        let color = current.level.color();
        let pending = self.queue.len();
        let title = Line::from(vec![
            " ".into(),
            current.level.marker_span(),
            " ".into(),
            current.level.label().bold().fg(color),
            if pending > 1 {
                format!(" ({pending} in wachtrij) ").into()
            } else {
                " ".into()
            },
        ]);

        let block = Block::bordered()
            .title(title)
            .title_alignment(Alignment::Center)
            .border_set(border::THICK)
            .border_style(Style::default().fg(color));
        let inner = block.inner(area);
        block.render(area, buf);

        if matches!(current.level, Level::Critical) {
            let throbber = Throbber::default()
                .label(current.message.clone())
                .style(Style::default().fg(color))
                .throbber_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
                .throbber_set(throbber_widgets_tui::BLACK_CIRCLE)
                .use_type(throbber_widgets_tui::WhichUse::Spin);
            StatefulWidget::render(throbber, inner, buf, state);
        } else {
            Paragraph::new(current.message.clone())
                .wrap(Wrap { trim: true })
                .render(inner, buf);
        }
    }
}
