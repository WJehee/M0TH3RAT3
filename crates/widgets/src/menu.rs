use ratatui::{
    prelude::*,
    widgets::{List, ListState, StatefulWidget},
};

/// Vertical menu that distinguishes the highlighted item (cursor) from the
/// active one (the screen currently shown). Generic over the item type so
/// each app can use its own screen enum.
#[derive(Debug)]
pub struct Menu<T> {
    items: Vec<T>,
    list_state: ListState,
    selected: usize,
    active: usize,
}

impl<T: Copy + PartialEq> Menu<T> {
    pub fn new(items: Vec<T>) -> Self {
        assert!(!items.is_empty(), "a menu needs at least one item");
        Self {
            items,
            list_state: ListState::default().with_selected(Some(0)),
            selected: 0,
            active: 0,
        }
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn selected(&self) -> T {
        self.items[self.selected]
    }

    pub fn active(&self) -> T {
        self.items[self.active]
    }

    /// Moves the cursor by `offset`, wrapping around at both ends.
    pub fn select(&mut self, offset: isize) {
        let len = self.items.len() as isize;
        self.selected = ((self.selected as isize + offset).rem_euclid(len)) as usize;
        self.list_state.select(Some(self.selected));
    }

    pub fn activate(&mut self) {
        self.active = self.selected;
    }

    pub fn set_active(&mut self, item: T) {
        if let Some(i) = self.items.iter().position(|it| *it == item) {
            self.active = i;
            self.selected = i;
            self.list_state.select(Some(i));
        }
    }

    /// Renders the menu, letting the caller build each line so it can add
    /// notification markers or strike through unavailable screens.
    pub fn render<'a>(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        line_for: impl Fn(&T) -> Line<'a>,
    ) {
        let lines: Vec<Line<'a>> = self.items.iter().map(line_for).collect();
        let list = List::new(lines)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().bold().fg(Color::Green))
            .repeat_highlight_symbol(true);
        StatefulWidget::render(list, area, buf, &mut self.list_state);
    }
}
