use ratatui::{prelude::*, symbols::border, widgets::*};

const YODA: &str = r#"
                   ____                  
                _.' :  `._               
            .-.'`.  ;   .'`.-.           
   __      / : ___\ ;  /___ ; \      __  
 ,'_ ""--.:__;".-.";: :".-.":__;.--"" _`,
 :' `.t""--.. '<@.`;_  ',@>` ..--""j.' `;
      `:-.._J '-.-'L__ `-- ' L_..-;'     
        "-.__ ;  .-"  "-.  : __.-"       
            L ' /.------.\ ' J           
             "-.   "--"   .-"            
            __.l"-:_JL_;-";.__           
         .-j/'.;  ;""""  / .'\"-.        
       .' /:`. "-.:     .-" .';  `.      
    .-"  / ;  "-. "-..-" .-"  :    "-.   
 .+"-.  : :      "-.__.-"      ;-._   \  
 ; \  `.; ;                    : : "+. ; 
 :  ;   ; ;                    : ;  : \: 
"#;

const BOB: &str = r#"
      .--..--..--..--..--..--.
    .' \  (`._   (_)     _   \
  .'    |  '._)         (_)  |
  \ _.')\      .----..---.   /
  |(_.'  |    /    .-\-.  \  |
  \     0|    |   ( O| O) | o|
   |  _  |  .--.____.'._.-.  |
   \ (_) | o         -` .-`  |
    |    \   |`-._ _ _ _ _\ /
    \    |   |  `. |_||_|   |
    | o  |    \_      \     |     -.   .-.
    |.-.  \     `--..-'   O |     `.`-' .'
  _.'  .' |     `-.-'      /-.__   ' .-'
.' `-.` '.|='=.='=.='=.='=|._/_ `-'.'
`-._  `.  |________/\_____|    `-.'
   .'   ).| '=' '='\/ '=' |
   `._.`  '---------------'
           //___\   //___\
             ||       ||
             ||_.-.   ||_.-.
            (_.--__) (_.--__)
"#;

const SKELETON: &str = r#"
      .-.
     (o.o)
      |=|
     __|__
   //.=|=.\\
  // .=|=. \\
  \\ .=|=. //
   \\(_=_)//
    (:| |:)
     || ||
     () ()
     || ||
     || ||
    ==' '==
"#;

const ALIEN: &str = r#"
 o            o
  \          /
   \        /
    :-'""'-:
 .-'  ____  `-.
( (  (_()_)  ) )
 `-.   ^^   .-'
    `._==_.'
     __)(___
"#;

const XENOMORPH: &str = r#"
         __.,,------.._
      ,'"   _      _   "`.
     /.__, ._  -=- _"`    Y
    (.____.-.`      ""`   j
     VvvvvvV`.Y,.    _.,-'       ,     ,     ,
        Y    ||,   '"\         ,/    ,/    ./
        |   ,'  ,     `-..,'_,'/___,'/   ,'/   ,
   ..  ,;,,',-'"\,'  ,  .     '     ' ""' '--,/    .. ..
 ,'. `.`---'     `, /  , Y -=-    ,'   ,   ,. .`-..||_|| ..
ff\\`. `._        /f ,'j j , ,' ,   , f ,  \=\ Y   || ||`||_..
l` \` `.`."`-..,-' j  /./ /, , / , / /l \   \=\l   || `' || ||...
 `  `   `-._ `-.,-/ ,' /`"/-/-/-/-"'''"`.`.  `'.\--`'--..`'_`' || ,
            "`-_,',  ,'  f    ,   /      `._    ``._     ,  `-.`'//         ,
          ,-"'' _.,-'    l_,-'_,,'          "`-._ . "`. /|     `.'\ ,       |
        ,',.,-'"          \=) ,`-.         ,    `-'._`.V |       \ // .. . /j
        |f\\               `._ )-."`.     /|         `.| |        `.`-||-\\/
        l` \`                 "`._   "`--' j          j' j          `-`---'
         `  `                     "`,-  ,'/       ,-'"  /
                                 ,'",__,-'       /,, ,-'
                                 Vvv'            VVv'
"#;

const GUY: &str = r#"
   .------\ /------.
   |       -       |
   |               |
   |               |
   |               |
_______________________
===========.===========
  / ~~~~~     ~~~~~ \
 /|     |     |\
 W   ---  / \  ---   W
 \.      |o o|      ./
  |                 |
  \    #########    /
   \  ## ----- ##  /
    \##         ##/
     \_____v_____/
"#;

struct CrewMember {
    name: String,
    picture: String,
    role: String,
    location: String,
    vitality: u8,
}

impl CrewMember {
    pub fn new(name: String, picture: String, role: String) -> CrewMember {
        CrewMember {
            name,
            picture,
            role,
            location: String::from("???"),
            vitality: 1,
        }
    }
}

impl Widget for &CrewMember {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // let [left, right] = Layout::horizontal([
        //     Constraint::Percentage(50),
        //     Constraint::Percentage(50),
        // ])
        //     .areas(area);

        // let info = Paragraph::new(self.name.clone());
        // info.render(left, buf);
 
        let picture = Paragraph::new(self.picture.clone());
        picture.render(area, buf);

        let block = Block::bordered()
            .title_bottom(self.name.clone().bold())
            .title_alignment(Alignment::Center)
            .border_set(border::PLAIN);
        
        block.render(area, buf);
    }
}

pub struct CrewStatus;

impl Widget for &CrewStatus {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [col_left, col_right ]= Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]).areas(area);

        let cards = Layout::vertical([
            Constraint::Ratio(1, 3) ; 3
        ]);
        let [top_left, mid_left, bot_left] = cards.areas(col_left);
        let [top_right, mid_right, bot_right] = cards.areas(col_right);

        CrewMember::new(String::from("Yoda"), String::from(YODA), String::new()).render(top_left, buf);
        CrewMember::new(String::from("Jack Skellington"), String::from(SKELETON), String::new()).render(top_right, buf);
        CrewMember::new(String::from("???"), String::from(XENOMORPH), String::new()).render(mid_left, buf);
        CrewMember::new(String::from("Bob"), String::from(BOB), String::new()).render(mid_right, buf);
        CrewMember::new(String::from("Yabooiiii"), String::from(GUY), String::new()).render(bot_left, buf);
        CrewMember::new(String::from("Ally"), String::from(ALIEN), String::new()).render(bot_right, buf);
    }   
}

