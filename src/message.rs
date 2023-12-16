#![allow(dead_code)]
use std::fmt::{
    Error,
    Display,
    Formatter,
};

pub const SOCKET_ADDR: &'static str = "/run/user/1000/liver.sock";
pub const USAGE:       &'static str =
"
Usage:
    queue [class] <motion>  queue <motion> from <class>
    set [class] <motion>    set <motion> from <class>
    play                    resume animation
    pause                   pause animation
    toggle                  toggle animation
    exit                    exit the application
    help                    print this info and quit
";

//  __  __
// |  \/  | ___  ___ ___  __ _  __ _  ___
// | |\/| |/ _ \/ __/ __|/ _` |/ _` |/ _ \
// | |  | |  __/\__ \__ \ (_| | (_| |  __/
// |_|  |_|\___||___/___/\__,_|\__, |\___|
//                             |___/

pub enum Message {
    SetMotion((String, String)),
    QueueMotion((String, String)),
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
    pub fn parse(input: String) -> Option<Self>
    {
        let mut message = input.split(':');

        match message.next() {
            Some(action) => match action {
                "toggle" => Some(Message::Toggle),
                "pause"  => Some(Message::Pause),
                "play"   => Some(Message::Play),
                "exit"   => Some(Message::Exit),
                "queue"  => {
                    let first = match message.next() {
                        Some(f) => f,
                        None    => return None
                    };
                    let result = match message.next() {
                        Some(second) => (first.to_string(),
                                         second.to_string()),
                        None         => ("".to_string(),
                                         first.to_string())
                    };

                    Some(Message::QueueMotion(result))
                }
                "set"    => {
                    let first = match message.next() {
                        Some(f) => f,
                        None    => return None
                    };
                    let result = match message.next() {
                        Some(second) => (first.to_string(),
                                         second.to_string()),
                        None         => ("".to_string(),
                                         first.to_string())
                    };

                    Some(Message::SetMotion(result))
                }
                _ => None
            }
            None => None
        }
    }
}

impl Display for Message {
    fn fmt(&self,
           f: &mut Formatter<'_>) -> Result<(), Error>
    {
        match self {
            Message::SetMotion(s)   => write!(f, "set:{}:{}", s.0, s.1),
            Message::QueueMotion(s) => write!(f, "queue:{}:{}", s.0, s.1),
            Message::Toggle         => write!(f, "toggle:"),
            Message::Pause          => write!(f, "pause:"),
            Message::Play           => write!(f, "play:"),
            Message::Exit           => write!(f, "exit:"),
        }
    }
}

