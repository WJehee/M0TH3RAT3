use num_traits::abs;
use ratatui::layout::{Constraint, Flex, Layout, Rect};

use crate::objects::SolarSystem;

pub const WARP_HOLD_DURATION: u64 = 1;
pub const TITLE_HEADER: &str = r#"
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

pub enum Event {
    Item(ItemDiff),
    NewSystem(Option<SolarSystem>),
    PlanetUpdate,
    RandomEvent,
}

pub struct ItemDiff {
    pub crystals: i32,
    pub fuel: i32,
    pub components: i32,
}

pub fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

pub fn distance(pos1: (f64, f64), pos2: (f64, f64)) -> f64 {
    let x_distance = abs(pos1.0 - pos2.0);
    let y_distance = abs(pos1.1 - pos2.1);
    ((x_distance * x_distance) + (y_distance * y_distance)).sqrt()
}

pub fn within_radius(pos1: (f64, f64), pos2: (f64, f64), radius: f64) -> bool {
    distance(pos1, pos2) <= radius
}


/// Small xorshift64 generator so the simulation modules do not need the rand
/// crate. Statistical quality is irrelevant here; it only feeds fake data.
pub struct XorShift(u64);

impl XorShift {
    pub fn new(seed: u64) -> Self {
        // Xorshift gets stuck at zero forever, so a zero seed is replaced.
        Self(if seed == 0 { 0x9E3779B97F4A7C15 } else { seed })
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Uniform in [0, 1).
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64)
    }

    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.unit()
    }

    /// Approximately standard normal, via the sum of twelve uniforms. The sum
    /// has variance exactly 1 and the tails are clipped at +-6, which is fine
    /// for animating prices.
    pub fn gaussian(&mut self) -> f64 {
        (0..12).map(|_| self.unit()).sum::<f64>() - 6.0
    }
}
