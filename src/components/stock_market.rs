//! Interstellar stock exchange screen.
//!
//! Prices are simulated with a geometric random walk (log-price plus a Gaussian
//! step) so that they stay positive and behave roughly like real quotes. On top
//! of the walk, rare "shocks" multiply a price by a random factor to produce
//! crashes and rallies worth a headline. The whole thing is fake data for now;
//! there is no trading yet, only watching.
use std::collections::VecDeque;

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    symbols,
    widgets::{Axis, Block, Chart, Dataset, GraphType, Paragraph, Row, Table},
};

use crate::util::XorShift;

/// Number of price samples kept per company, which is also the chart width in
/// data points. Braille markers render two points per column, so 120 samples
/// fill a chart of roughly 60 columns, which is about what the main panel
/// offers on a 100 column terminal.
const HISTORY_LEN: usize = 120;

/// The app ticks every 200ms, which is too frantic for stock quotes, so prices
/// only advance every fifth tick: once per second. The ticker tape still
/// scrolls on every tick so it stays smooth.
const TICKS_PER_UPDATE: u64 = 5;

/// Chance per price update that a company suffers a shock. With ten companies
/// updating once per second this yields a headline about every 40 seconds,
/// which keeps the notification panel interesting without spamming it.
const SHOCK_CHANCE: f64 = 0.0025;

/// Prices cannot go to zero because the log-walk would never recover from it,
/// so a floor keeps every company at least nominally alive.
const PRICE_FLOOR: f64 = 0.01;

struct CompanySpec {
    ticker: &'static str,
    name: &'static str,
    open: f64,
    /// Expected log-return per tick. Tiny values, positive for growers,
    /// negative for companies that are slowly going under.
    drift: f64,
    /// Standard deviation of the log-return per tick.
    volatility: f64,
}

/// Corporations from dystopian science fiction, mostly film. The drift and
/// volatility loosely follow their reputation: Weyland-Yutani is a stable
/// blue chip, Tyrell is volatile after the whole replicant affair, and Soylent
/// keeps growing because people keep eating.
const COMPANIES: [CompanySpec; 10] = [
    CompanySpec { ticker: "WYC",  name: "Weyland-Yutani Corp",      open: 412.50, drift: 0.0002,  volatility: 0.006 },
    CompanySpec { ticker: "TYRL", name: "Tyrell Corporation",       open: 187.20, drift: -0.0003, volatility: 0.018 },
    CompanySpec { ticker: "WLLC", name: "Wallace Corporation",      open: 256.80, drift: 0.0004,  volatility: 0.010 },
    CompanySpec { ticker: "OCP",  name: "Omni Consumer Products",   open: 98.40,  drift: 0.0001,  volatility: 0.012 },
    CompanySpec { ticker: "CYBD", name: "Cyberdyne Systems",        open: 143.10, drift: 0.0005,  volatility: 0.015 },
    CompanySpec { ticker: "UMBR", name: "Umbrella Corporation",     open: 64.75,  drift: -0.0002, volatility: 0.020 },
    CompanySpec { ticker: "BNL",  name: "Buy n Large",              open: 33.20,  drift: 0.0003,  volatility: 0.008 },
    CompanySpec { ticker: "RKL",  name: "Rekall Inc",               open: 21.90,  drift: 0.0000,  volatility: 0.025 },
    CompanySpec { ticker: "SOYL", name: "Soylent Corporation",      open: 12.45,  drift: 0.0006,  volatility: 0.009 },
    CompanySpec { ticker: "UAC",  name: "Union Aerospace Corp",     open: 77.60,  drift: -0.0001, volatility: 0.014 },
];

struct Company {
    spec: &'static CompanySpec,
    history: VecDeque<f64>,
}

impl Company {
    fn price(&self) -> f64 {
        *self.history.back().expect("history is never empty")
    }

    /// Change relative to the opening price, in percent.
    fn change_pct(&self) -> f64 {
        (self.price() / self.spec.open - 1.0) * 100.0
    }
}

pub struct StockMarket {
    companies: Vec<Company>,
    selected: usize,
    rng: XorShift,
    /// Tick counter driving the scrolling ticker tape at the bottom.
    ticks: u64,
}

impl Default for StockMarket {
    fn default() -> Self {
        Self::new()
    }
}

impl StockMarket {
    pub fn new() -> Self {
        let mut market = Self {
            companies: COMPANIES
                .iter()
                .map(|spec| Company {
                    spec,
                    history: VecDeque::from(vec![spec.open]),
                })
                .collect(),
            selected: 0,
            rng: XorShift::new(0xD1B54A32D192ED03),
            ticks: 0,
        };
        // Pre-fill the history so the chart is not empty on the first frame.
        // Shocks are skipped here because a headline for something that
        // happened before the screen was opened would be confusing.
        for _ in 0..HISTORY_LEN {
            for i in 0..market.companies.len() {
                market.step(i);
            }
        }
        market
    }

    fn step(&mut self, i: usize) {
        let spec = self.companies[i].spec;
        let z = self.rng.gaussian();
        let next = (self.companies[i].price() * (spec.drift + spec.volatility * z).exp())
            .max(PRICE_FLOOR);
        self.push_price(i, next);
    }

    fn push_price(&mut self, i: usize, price: f64) {
        let history = &mut self.companies[i].history;
        if history.len() == HISTORY_LEN {
            history.pop_front();
        }
        history.push_back(price);
    }

    /// Called on every app tick. Prices advance only once per
    /// `TICKS_PER_UPDATE` ticks. Returns a headline when a company was hit by
    /// a shock, so the caller can show it as a notification.
    pub fn tick(&mut self) -> Option<String> {
        self.ticks += 1;
        if self.ticks % TICKS_PER_UPDATE != 0 {
            return None;
        }
        let mut headline = None;
        for i in 0..self.companies.len() {
            self.step(i);
            if self.rng.unit() < SHOCK_CHANCE {
                // Factors below 1 are crashes, above 1 rallies. The range is
                // asymmetric so a crash can wipe out half the value but a
                // rally only adds a third; markets fall faster than they rise.
                let factor = if self.rng.unit() < 0.5 {
                    self.rng.range(0.5, 0.85)
                } else {
                    self.rng.range(1.10, 1.35)
                };
                let price = (self.companies[i].price() * factor).max(PRICE_FLOOR);
                self.push_price(i, price);
                let spec = self.companies[i].spec;
                headline = Some(if factor < 1.0 {
                    format!("{} stort in: {:.0}% verlies", spec.ticker, (1.0 - factor) * 100.0)
                } else {
                    format!("{} schiet omhoog: +{:.0}%", spec.ticker, (factor - 1.0) * 100.0)
                });
            }
        }
        headline
    }

    pub fn handle_press_event(&mut self, key_event: KeyEvent) {
        let n = self.companies.len();
        match key_event.code {
            KeyCode::Left => { self.selected = (self.selected + n - 1) % n; }
            KeyCode::Right | KeyCode::Tab => { self.selected = (self.selected + 1) % n; }
            _ => {}
        }
    }

    fn ticker_tape(&self, width: usize) -> String {
        let tape: String = self
            .companies
            .iter()
            .map(|c| {
                let arrow = if c.change_pct() >= 0.0 { "▲" } else { "▼" };
                format!("{} {:.2} {}{:.1}%   ", c.spec.ticker, c.price(), arrow, c.change_pct().abs())
            })
            .collect();
        let chars: Vec<char> = tape.chars().collect();
        if chars.is_empty() || width == 0 {
            return String::new();
        }
        let offset = (self.ticks as usize) % chars.len();
        chars.iter().cycle().skip(offset).take(width).collect()
    }
}

fn change_color(pct: f64) -> Color {
    if pct >= 0.0 { Color::Green } else { Color::Red }
}

impl Widget for &StockMarket {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [chart_area, table_area, tape_area] = Layout::vertical([
            Constraint::Percentage(55),
            Constraint::Percentage(45),
            Constraint::Length(1),
        ])
        .areas(area);

        self.render_chart(chart_area, buf);
        self.render_table(table_area, buf);

        Paragraph::new(self.ticker_tape(tape_area.width as usize))
            .style(Style::default().fg(Color::Yellow))
            .render(tape_area, buf);
    }
}

impl StockMarket {
    fn render_chart(&self, area: Rect, buf: &mut Buffer) {
        let company = &self.companies[self.selected];
        let data: Vec<(f64, f64)> = company
            .history
            .iter()
            .enumerate()
            .map(|(i, v)| (i as f64, *v))
            .collect();

        let (min, max) = company
            .history
            .iter()
            .fold((f64::MAX, f64::MIN), |(lo, hi), v| (lo.min(*v), hi.max(*v)));
        // A little headroom so the line never touches the frame, and a
        // non-zero span so a perfectly flat history still renders.
        let pad = ((max - min) * 0.1).max(0.5);
        let (lo, hi) = (min - pad, max + pad);

        let pct = company.change_pct();
        let color = change_color(pct);
        let dataset = Dataset::default()
            .name(company.spec.ticker)
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(color))
            .data(&data);

        let title = Line::from(vec![
            format!(" {} ", company.spec.name).bold(),
            format!("{:.2} ", company.price()).into(),
            format!("{}{:.2}% ", if pct >= 0.0 { "+" } else { "" }, pct).fg(color),
        ]);

        let y_axis = Axis::default()
            .bounds([lo, hi])
            .labels(vec![
                format!("{:.1}", lo),
                format!("{:.1}", (lo + hi) / 2.0),
                format!("{:.1}", hi),
            ])
            .style(Style::default().fg(Color::DarkGray));
        let x_axis = Axis::default()
            .bounds([0.0, HISTORY_LEN as f64])
            .style(Style::default().fg(Color::DarkGray));

        Chart::new(vec![dataset])
            .block(
                Block::bordered()
                    .title(title)
                    .title_alignment(Alignment::Center)
                    .title_bottom(Line::from(vec![
                        " Vorige ".into(),
                        "<Left>".green().bold(),
                        " Volgende ".into(),
                        "<Right> ".green().bold(),
                    ])),
            )
            .x_axis(x_axis)
            .y_axis(y_axis)
            .render(area, buf);
    }

    fn render_table(&self, area: Rect, buf: &mut Buffer) {
        let rows: Vec<Row> = self
            .companies
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let pct = c.change_pct();
                let mut row = Row::new(vec![
                    Line::from(c.spec.ticker),
                    Line::from(c.spec.name),
                    Line::from(format!("{:.2}", c.price())).right_aligned(),
                    Line::from(format!("{}{:.2}%", if pct >= 0.0 { "+" } else { "" }, pct))
                        .right_aligned()
                        .fg(change_color(pct)),
                ]);
                if i == self.selected {
                    row = row.style(Style::default().bold().fg(Color::Green).reversed());
                }
                row
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(5),
                Constraint::Min(16),
                Constraint::Length(10),
                Constraint::Length(9),
            ],
        )
        .header(
            Row::new(vec![
                Line::from("CODE"),
                Line::from("BEDRIJF"),
                Line::from("KOERS").right_aligned(),
                Line::from("VERSCHIL").right_aligned(),
            ])
            .style(Style::default().fg(Color::DarkGray).bold()),
        )
        .block(
            Block::bordered()
                .title("KOERSEN".bold())
                .title_alignment(Alignment::Center),
        );
        Widget::render(table, area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_without_panicking() {
        let mut market = StockMarket::new();
        for _ in 0..500 {
            market.tick();
        }
        market.handle_press_event(KeyEvent::from(KeyCode::Right));
        let area = Rect::new(0, 0, 78, 36);
        let mut buf = Buffer::empty(area);
        (&market).render(area, &mut buf);
        for y in 0..area.height {
            let line: String = (0..area.width)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect();
            println!("{line}");
        }
        assert_eq!(market.selected, 1);
    }
}
