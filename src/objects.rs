use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct SolarSystem {
    name: String,
    x: f64,
    y: f64,
    planets: Vec<Planet>,
}

#[derive(Deserialize, Serialize, Clone)]
struct Planet {
    // Visual traits
    name: String,
    x: f64,
    y: f64,
    radius: f64,
    // color: Color,

    // Random traits
    diameter_km: u32,
    temp: i32,
    gravity: f32,
    hours_per_day: u32,
    tags: Vec<PlanetTag>
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

