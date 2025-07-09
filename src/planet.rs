use std::fmt;

enum PlanetSize {
    Small,
    Medium,
    Large,
    Huge,
}

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


struct Planet {
    name: String,
    // Random traits
    diameter_km: u32,
    temp: i32,
    gravity: f32,
    hours_per_day: u32,
    tags: Vec<PlanetTag>
}

