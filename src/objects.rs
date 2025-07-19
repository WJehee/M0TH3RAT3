use std::{fmt};

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

use crate::components::star_map::{Location, StarMap};

#[derive(Deserialize, Serialize, Clone)]
pub struct SolarSystem {
    pub name: String,
    pub pos: (f64, f64),
    pub planets: Vec<Planet>,
}

impl SolarSystem {
    pub fn to_star_map(self) -> StarMap {
        let mut locations = Vec::new();
        let p1 = Location::new(50.0, 80.0, 5.0, Color::Magenta);
        let p2 = Location::new(30.0, 20.0, 8.0, Color::Red);
        let p3 = Location::new(80.0, 30.0, 3.0, Color::LightGreen);
        locations.push(p1);
        locations.push(p2);
        locations.push(p3);
        StarMap::new(locations)
    }

    pub fn has_component(self) -> bool {
        for planet in self.planets {
            if planet.has_component { return true; }
        }
        false
    }

    pub fn has_event(self) -> bool {
        for planet in self.planets {
            if planet.has_event{ return true; }
        }
        false
    }
}

#[derive(Deserialize, Serialize, Clone)]
struct Planet {
    // Visual traits
    name: String,
    x: f64,
    y: f64,
    radius: f64,

    // Random info 
    diameter_km: u32,
    temp: i32,
    gravity: f32,
    hours_per_day: u32,
    tags: Vec<PlanetTag>,

    // Game related
    has_event: bool,
    has_component: bool,
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

