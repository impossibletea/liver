use std::{
    env,
    io::Write,
    error::Error,
    os::unix::net::UnixStream,
};

mod message;
use message::{Message, SOCKET_ADDR};

fn main() -> Result<(), Box<dyn Error>>
{
    let mut args = env::args();
    args.next().expect("the first argument to be the executable");

    let mut stream =
        UnixStream::connect(SOCKET_ADDR)
        .map_err(|e| format!("Failed to connect to socket: {e}"))?;

    if let Some(arg) = args.next() {
        match arg.as_str() {
            "toggle" => write!(&mut stream, "{}", Message::Toggle)?,
            "pause"  => write!(&mut stream, "{}", Message::Pause)?,
            "play"   => write!(&mut stream, "{}", Message::Play)?,
            "exit"   => write!(&mut stream, "{}", Message::Exit)?,
            "set"    => {
                let first =
                    args.next()
                    .ok_or("What motion to set?".to_string())?;

                let result = match args.next() {
                    Some(second) => (first,
                                     second),
                    None         => ("".to_string(),
                                     first)
                };

                write!(&mut stream, "{}", Message::SetMotion(result))?;
            },
            "queue"  => {
                let first =
                    args.next()
                    .ok_or("What motion to set?".to_string())?;

                let result = match args.next() {
                    Some(second) => (first,
                                     second),
                    None         => ("".to_string(),
                                     first)
                };

                write!(&mut stream, "{}", Message::QueueMotion(result))?;
            },
            "help"   => println!("{}", message::USAGE),
            _        => eprintln!("Command `{}` is not recognised", arg)
        }
    } else {eprintln!("{}", message::USAGE)};

    Ok(())
}

