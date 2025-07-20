use std::{collections::HashSet, fmt};

use ratatui::{prelude::*, style::Color, widgets::canvas::{Circle, Context}};
use serde::{Deserialize, Serialize};

use crate::{components::star_map::StarMap, util::{Event, ItemDiff}};

#[derive(Deserialize, Serialize, Clone)]
pub struct SolarSystem {
    pub name: String,
    pub pos: (f64, f64),
    pub planets: Vec<Planet>,
}

impl SolarSystem {
    pub fn to_star_map(&self) -> StarMap {
        StarMap::new(self.planets.clone())
    }

    pub fn has_component(&self) -> bool {
        for planet in &self.planets {
            if planet.has_component { return true; }
        }
        false
    }

    pub fn has_event(&self) -> bool {
        for planet in &self.planets {
            if planet.has_event{ return true; }
        }
        false
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Planet {
    // Visual traits
    name: String,
    x: f64,
    y: f64,
    radius: f64,

    // TODO: Random info, show in popup
    // diameter_km: u32,
    // temp: i32,
    // gravity: f32,
    // hours_per_day: u32,
    // tags: Vec<PlanetTag>,

    // Game related
    has_event: bool,
    has_component: bool,
    crystals: i32,
    fuel: i32,
    visited_by: HashSet<String>,
}

impl Planet {
    pub fn visit(&mut self, name: String) -> Vec<Event> {
        self.visited_by.insert(name);
        let mut events = Vec::new();
        let mut diff = ItemDiff {
            crystals: self.crystals,
            fuel: self.fuel,
            components: 0,
        };
        self.crystals = 0;
        self.fuel = 0;
        if self.has_event {
            self.has_event = false;
            events.push(Event::RandomEvent);
        }
        if self.has_component {
            self.has_component = false;
            diff.components += 1;
        }
        events.push(Event::Item(diff));
        events.push(Event::PlanetUpdate);
        events
    }

    pub fn draw(&self, ctx: &mut Context, highlighted: Option<Color>) {
        let mut i = self.radius;
        while i > 0.0 {
            ctx.draw(&Circle {
                x: self.x,
                y: self.y,
                radius: i,
                color: self.get_color(),
            });
            i -= 0.5;
        }
        ctx.draw(&Circle {
            x: self.x,
            y: self.y,
            radius: self.radius * 1.7,
            color: highlighted.unwrap_or(Color::DarkGray),
        });
    }

    pub fn draw_current(&self, ctx: &mut Context) {
        ctx.print(
            self.x-(self.radius/2.0), self.y+(self.radius*2.0),
            "Jij bent hier".green().bold()
            // "You are here".green().bold()
        );
    }

    fn get_color(&self) -> Color {
        Color::Red
    }
}

#[derive(Deserialize, Serialize, Clone)]
enum PlanetSize {
    Small,
    Medium,
    Large,
    Huge,
}

#[derive(Deserialize, Serialize, Clone)]
enum PlanetTag {
    Gas,
    Terrestrial,
    Ocean,
}

impl fmt::Display for PlanetTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res = match self {
            PlanetTag::Gas => "Gas",
            PlanetTag::Terrestrial => "Aards",
            PlanetTag::Ocean => "Oceaan",
        };
        write!(f, "{}", res)
    }
}

