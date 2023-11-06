use std::fmt::{
    Error,
    Display,
    Formatter,
};

pub const SOCKET_ADDR: &'static str = "/run/user/1000/rusty-ships.sock";

//  __  __
// |  \/  | ___  ___ ___  __ _  __ _  ___
// | |\/| |/ _ \/ __/ __|/ _` |/ _` |/ _ \
// | |  | |  __/\__ \__ \ (_| | (_| |  __/
// |_|  |_|\___||___/___/\__,_|\__, |\___|
//                             |___/

pub enum Message {
    SetMotion(String),
    Toggle,
    Pause,
    Play,
    Exit,
}

//  __  __
// |  \/  | ___  ___ ___  __ _  __ _  ___   _ _
// | |\/| |/ _ \/ __/ __|/ _` |/ _` |/ _ \ (_|_)
// | |  | |  __/\__ \__ \ (_| | (_| |  __/  _ _
// |_|  |_|\___||___/___/\__,_|\__, |\___| (_|_)
//                             |___/

impl Message {
    #[allow(dead_code)]
    pub fn parse(input: String) -> Option<Self> {
        let mut message = input.split(':');

        match message.next() {
            Some(action) => match action {
                "toggle" => Some(Message::Toggle),
                "pause"  => Some(Message::Pause),
                "play"   => Some(Message::Play),
                "exit"   => Some(Message::Exit),
                "set"    => if let Some(motion) = message.next() {
                    Some(Message::SetMotion(motion.to_string()))
                } else {None}
                _ => None
            }
            None => None
        }
    }
}

impl Display for Message {
    fn fmt(&self,
           f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Message::SetMotion(s) => write!(f, "set:{}", s),
            Message::Toggle       => write!(f, "toggle:"),
            Message::Pause        => write!(f, "pause:"),
            Message::Play         => write!(f, "play:"),
            Message::Exit         => write!(f, "exit:"),
        }
    }
}

