use std::{
    env,
    io::Write,
    os::unix::net::UnixStream,
};

mod message;
use message::{Message, SOCKET_ADDR};

fn main() -> Result<(), String> {
    let mut args = env::args();
    args.next().expect("the first argument to be the executable");

    let mut stream =
        UnixStream::connect(SOCKET_ADDR)
        .map_err(|e| format!("Failed to connect to socket: {e}"))?;

    match args.next() {
        Some(arg) => match arg.as_str() {
            "toggle" => write!(&mut stream, "{}", Message::Toggle)
                       .map_err(|e| format!("Failed to send message: {e}")),
            "pause"  => write!(&mut stream, "{}", Message::Pause)
                        .map_err(|e| format!("Failed to send message: {e}")),
            "play"   => write!(&mut stream, "{}", Message::Play)
                        .map_err(|e| format!("Failed to send message: {e}")),
            "exit"   => write!(&mut stream, "{}", Message::Exit)
                        .map_err(|e| format!("Failed to send message: {e}")),
            "set"    => {
                let motion =
                    args.next()
                    .ok_or(format!("What motion to set?"))?;

                write!(&mut stream, "{}", Message::SetMotion(motion))
                .map_err(|e| format!("Failed to send message: {e}"))
            },
            _        => Err(format!("Command {} is not recognised", arg))
        }
        None => Err(format!("No command provided"))
    }
}

