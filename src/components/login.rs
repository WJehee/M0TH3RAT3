use ratatui::prelude::*;
use ratatui::widgets::Widget;

use crate::DB;

struct User {
    username: String,
    password: String,
}

pub fn login_user(username: &str, password: &str) -> Option<User> {
    let db = DB.lock().unwrap();
    let pw: String = db.query_one("SELECT password FROM users WHERE username=?", ((username),), |row| row.get(0)).ok()?;

    if pw != password { return None }

    let user = User {
        username: String::from("bob"),
        password: String::from("bob"),
    };

    Some(user)
}

struct LoginScreen;

// impl LoginScreen {
//     pub fn handle_press_event(&mut self, key_event: KeyEvent) {
//     }
// }

// impl Widget for &LoginScreen {
//     pub fn render(self, area: Rect, buf: &mut Buffer) {
//
//     }
// }

