use ratatui::layout::{Constraint, Flex, Layout, Rect};

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
