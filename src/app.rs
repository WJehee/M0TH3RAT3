use std::io;

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind},
    symbols::border,
    widgets::{
        block::{Position, Title}, Block, List, ListState, Paragraph, Widget
    },
};

use num_traits::{FromPrimitive, ToPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};

use ratatui::prelude::*;

use crate::{components::{ship_status::ShipStatus, star_map::StarMap}, tui};

const TITLE_HEADER: &str = r#"
     _                      _______                      _      
  _dMMMb._              .adOOOOOOOOOba.              _,dMMMb_   
 dP'  ~YMMb            dOOOOOOOOOOOOOOOb            aMMP~  `Yb  
 V      ~"Mb          dOOOOOOOOOOOOOOOOOb          dM"~      V  
          `Mb.       dOOOOOOOOOOOOOOOOOOOb       ,dM'           
           `YMb._   |OOOOOOOOOOOOOOOOOOOOO|   _,dMP'            
      __     `YMMM| OP'~"YOOOOOOOOOOOP"~`YO |MMMP'     __       
    ,dMMMb.     ~~' OO     `YOOOOOP'     OO `~~     ,dMMMb.     
 _,dP~  `YMba_      OOb      `OOO'      dOO      _aMMP'  ~Yb._  
             `YMMMM\`OOOo     OOO     oOOO'/MMMMP'              
     ,aa.     `~YMMb `OOOb._,dOOOb._,dOOO'dMMP~'       ,aa.     
   ,dMYYMba._         `OOOOOOOOOOOOOOOOO'          _,adMYYMb.   
  ,MP'   `YMMba._      OOOOOOOOOOOOOOOOO       _,adMMP'   `YM.  
  MP'        ~YMMMba._ YOOOOPVVVVVYOOOOP  _,adMMMMP~       `YM  
  YMb           ~YMMMM\`OOOOI`````IOOOOO'/MMMMP~           dMP  
   `Mb.           `YMMMb`OOOI,,,,,IOOOO'dMMMP'           ,dM'   
     `'                  `OObNNNNNdOO'                   `'     
                           `~OOOOO~'                            

M0TH3R@3-OS
"#;

#[derive(Debug, Copy, Clone, FromPrimitive, ToPrimitive, strum::AsRefStr)]
enum MenuItem {
    StarMap = 0,
    Crew,
    Info,
}

#[derive(Debug)]
struct MenuState {
    list_state: ListState,
    selected: MenuItem,
    active: MenuItem,
}

impl MenuState {
    fn select(&mut self, offset: i8) {
        let current = self.selected as i8;
        let next = current + offset;
        if next == -1 { 
            // Set to last item in the list
            self.selected = MenuItem::Info
        } else {
            self.selected = match FromPrimitive::from_i8(next) {
                Some(d2) => d2,
                None => FromPrimitive::from_u8(0).unwrap(),
            };
        }
        self.list_state.select(self.selected.to_usize());
    }

    fn activate(&mut self) {
        self.active = self.selected;
    }
}

#[derive(Debug)]
pub struct App {
    exit: bool,
    menu: MenuState,
    starmap: StarMap,
}

impl App {
    pub fn new() -> Self {
        Self {
            exit: false,
            menu: MenuState {
                list_state: ListState::default().with_selected(Some(0)),
                selected: MenuItem::StarMap,
                active: MenuItem::StarMap,
            },
            starmap: StarMap::new(),
        }
    }

    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                self.handle_press_event(key);
                match self.menu.selected {
                    MenuItem::StarMap => { self.starmap.handle_press_event(key); },
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_press_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q')  => { self.exit = true; },
            KeyCode::Up         => { self.menu.select(-1); },
            KeyCode::Down       => { self.menu.select(1); },
            KeyCode::Enter      => { self.menu.activate(); },
            _ => {},
        }
    }

    fn render_title(&mut self, area: Rect, buf: &mut Buffer) {
        let instructions = Title::from(Line::from(vec![
            " Select ".into(),
            "<Enter>".green().bold(),
            " Move up ".into(),
            "<Up>".green().bold(),
            " Move down ".into(),
            "<Down>".green().bold(),
            " Quit ".into(),
            "<Q> ".green().bold(),
        ]));
        let block = Block::bordered()
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let text = Text::from(TITLE_HEADER)
            .fg(Color::Green);

        Paragraph::new(text)
            .centered()
            .block(block)
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let [_padding_top, menu_pos, _padding_bottom] = Layout::vertical([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ]).areas(area);

        let menu = List::new([
            Line::from(MenuItem::StarMap.as_ref()).alignment(Alignment::Center),
            Line::from(MenuItem::Crew.as_ref()).alignment(Alignment::Center),
            Line::from(MenuItem::Info.as_ref()).alignment(Alignment::Center),
        ])
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default()
                .bold()
                .fg(Color::Green)
            )
            .repeat_highlight_symbol(true);

        ratatui::prelude::StatefulWidget::render(menu, menu_pos, buf, &mut self.menu.list_state);
    }
}


impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [left, right] = Layout::horizontal([
            Constraint::Percentage(35),
            Constraint::Percentage(65),
        ]).areas(area);

        let [title, list, ship_status] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(35),
                Constraint::Percentage(25),
                Constraint::Percentage(40),
            ])
            .areas(left);

        self.render_title(title, buf);
        self.render_list(list, buf);
        ShipStatus.render(ship_status, buf);

        let title = Title::from(self.menu.active.as_ref().bold());
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .border_set(border::THICK);
        let inner = block.inner(right);
        block.render(right, buf);

        match self.menu.active {
            MenuItem::StarMap   => { self.starmap.render(inner, buf); },
            MenuItem::Crew      => {},
            MenuItem::Info      => {},
        }
    }
}

