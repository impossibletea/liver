use std::{
    process,
    error::Error,
    env::{self, Args},
    fmt::{Display, Formatter, Error as FE},
};

use super::{Config, FitConfig, BgType, constant::APP_NAME};

//   ____ _     ___      _    ____   ____ ____
//  / ___| |   |_ _|    / \  |  _ \ / ___/ ___|
// | |   | |    | |    / _ \ | |_) | |  _\___ \
// | |___| |___ | |   / ___ \|  _ <| |_| |___) |
//  \____|_____|___| /_/   \_\_| \_\\____|____/

const CLI_ARGS: &[Cli] = &[
    Cli {
        name:  "-size",
        help:  "Set initial window size",
        usage: "<width>[x<height>]",
        act:   cli_size,
    },
    Cli {
        name:  "-title",
        help:  "Set window title",
        usage: "<title>",
        act:   cli_title,
    },
    Cli {
        name:  "-fit",
        help:  "Set model fit type",
        usage: "{cover|contain}",
        act:   cli_fit,
    },
    Cli {
        name:  "-bg-variant",
        help:  "Set variant of background",
        usage: "{color|image}",
        act:   cli_bg_variant,
    },
    Cli {
        name:  "-bg-color",
        help:  "Set background color",
        usage: "RRGGBB[AA]",
        act:   cli_bg_color,
    },
    Cli {
        name:  "-bg-image",
        help:  "Path to background image",
        usage: "<path>",
        act:   cli_bg_image_path,
    },
    Cli {
        name:  "-name",
        help:  "Model configuration file (*.model3.json) in `model-path`",
        usage: "<file>",
        act:   cli_name,
    },
    Cli {
        name:  "-path",
        help:  "Path to model assets",
        usage: "<path>",
        act:   cli_path,
    },
    Cli {
        name:  "-motions-open",
        help:  "IDs of model motions to play on launch",
        usage: "<[class1:]motion1>,..",
        act:   cli_motions_open,
    },
    Cli {
        name:  "-motion-idle",
        help:  "ID of model motion to play on idle",
        usage: "<[class:]motion>",
        act:   cli_motion_idle,
    },
    Cli {
        name:  "-motion-usr1",
        help:  "ID of model motion to play on receiving SIGUSR1",
        usage: "<[class:]motion>",
        act:   cli_motion_usr1,
    },
    Cli {
        name:  "-help",
        help:  "Show this help message and exit",
        usage: "",
        act:   cli_help,
    },
];

//   ____ _ _
//  / ___| (_)
// | |   | | |
// | |___| | |
//  \____|_|_|

struct Cli {
    name:  &'static str,
    help:  &'static str,
    usage: &'static str,
    act:   Act,
}

type Act = fn(&mut Config,
              &mut Args) -> Result<(), Box<dyn Error>>;

impl Display for Cli {
    fn fmt(&self,
           f: &mut Formatter<'_>) -> Result<(), FE>
    {
        let usage = [self.name, self.usage].join(" ");
        write!(f, "{: <35} {}", usage, self.help)
    }
}

//             _ _
//  _ _    ___| (_)   __ _ _ __ __ _ ___
// (_|_)  / __| | |  / _` | '__/ _` / __|
//  _ _  | (__| | | | (_| | | | (_| \__ \
// (_|_)  \___|_|_|  \__,_|_|  \__, |___/
//                             |___/

pub fn cli_args(toml: &mut Config) -> Result<String, Box<dyn Error>>
{
    let mut args = env::args();
    let program = args.next().expect("program");

    while let Some(arg) = args.next() {
        match CLI_ARGS.iter().find(|cli| cli.name == arg) {
            Some(cli) => match (cli.act)(toml, &mut args) {
                Ok(a)  => a,
                Err(e) => {
                    eprintln!("{e}");
                    eprintln!("Usage: {}", cli.usage);
                    return Err(format!("Error parsing arguments").into())
                }
            }
            None => {
                eprintln!("Arguments:");
                CLI_ARGS.iter()
                .for_each(|cli| eprintln!("    {cli}"));
                return Err(format!("Unknown argument: {arg}").into())
            }
        }
    }

    Ok(program)
}

//        _          _
//  _ _  | |__   ___| |_ __
// (_|_) | '_ \ / _ \ | '_ \
//  _ _  | | | |  __/ | |_) |
// (_|_) |_| |_|\___|_| .__/
//                    |_|

fn cli_help(_: &mut Config,
            _: &mut Args) -> Result<(), Box<dyn Error>>
{
    println!("");
    println!("Usage:");
    println!("    {} [arguments]", APP_NAME);
    println!("");
    println!("Arguments:");
    CLI_ARGS.iter()
    .for_each(|cli| println!("    {cli}"));
    println!("");
    process::exit(0)
}

//            _
//  _ _   ___(_)_______
// (_|_) / __| |_  / _ \
//  _ _  \__ \ |/ /  __/
// (_|_) |___/_/___\___|

fn cli_size(c: &mut Config,
            a: &mut Args) -> Result<(), Box<dyn Error>>
{
    let xs =
        a.next()
        .ok_or("No window size provided")?;
    let mut xs = xs.split('x');

    let width =
        xs.next()
        .expect("width");
    let height =
        xs.next()
        .unwrap_or(width);

    c.window.size = [width.parse()?,
                     height.parse()?];

    Ok(())
}

//        _   _ _   _
//  _ _  | |_(_) |_| | ___
// (_|_) | __| | __| |/ _ \
//  _ _  | |_| | |_| |  __/
// (_|_)  \__|_|\__|_|\___|

fn cli_title(c: &mut Config,
             a: &mut Args) -> Result<(), Box<dyn Error>>
{
    c.window.title =
        a.next()
        .ok_or("No window title provided")?;

    Ok(())
}

//         __ _ _
//  _ _   / _(_) |_
// (_|_) | |_| | __|
//  _ _  |  _| | |_
// (_|_) |_| |_|\__|

fn cli_fit(c: &mut Config,
           a: &mut Args) -> Result<(), Box<dyn Error>>
{
    let fit =
        a.next()
        .ok_or("No fit type provided")?;

    c.window.fit = match fit.to_lowercase().as_str() {
        "cover"   => FitConfig::Cover,
        "contain" => FitConfig::Contain,
        _         => return Err(format!("Unknown fit type: {fit}").into())
    };

    Ok(())
}

//        _                             _             _
//  _ _  | |__   __ _  __   ____ _ _ __(_) __ _ _ __ | |_
// (_|_) | '_ \ / _` | \ \ / / _` | '__| |/ _` | '_ \| __|
//  _ _  | |_) | (_| |  \ V / (_| | |  | | (_| | | | | |_
// (_|_) |_.__/ \__, |   \_/ \__,_|_|  |_|\__,_|_| |_|\__|
//              |___/

fn cli_bg_variant(c: &mut Config,
                  a: &mut Args) -> Result<(), Box<dyn Error>>
{
    let var =
        a.next()
        .ok_or("No background variant provided")?;

    c.window.bg.variant = match var.to_lowercase().as_str() {
        "color" => BgType::Color,
        "image" => BgType::Image,
        _       => return Err(format!("Unknown background type: {var}").into())
    };

    Ok(())
}

//        _                       _
//  _ _  | |__   __ _    ___ ___ | | ___  _ __
// (_|_) | '_ \ / _` |  / __/ _ \| |/ _ \| '__|
//  _ _  | |_) | (_| | | (_| (_) | | (_) | |
// (_|_) |_.__/ \__, |  \___\___/|_|\___/|_|
//              |___/

fn cli_bg_color(c: &mut Config,
                a: &mut Args) -> Result<(), Box<dyn Error>>
{
    let color =
        a.next()
        .ok_or("No background color provided")?;

    let color = color.to_uppercase();

    let max =
        u16::from_str_radix("FF", 16)
        .expect("FF to be valid hex number");
    match color.len() {
        6 => [u16::from_str_radix(&color[0..2], 16)?,
              u16::from_str_radix(&color[2..4], 16)?,
              u16::from_str_radix(&color[4..6], 16)?,
              u16::from_str_radix(        "FF", 16)?],
        8 => [u16::from_str_radix(&color[0..2], 16)?,
              u16::from_str_radix(&color[2..4], 16)?,
              u16::from_str_radix(&color[4..6], 16)?,
              u16::from_str_radix(&color[6..8], 16)?],
        _ => return Err(format!("Incorrect color: {color}").into())
    }.into_iter()
    .map(|c| c as f32 / max as f32)
    .enumerate()
    .for_each(|(i, comp)| {
        dbg!(&comp);
        c.window.bg.color[i] = comp
    });

    Ok(())
}

//        _             _                                          _   _
//  _ _  | |__   __ _  (_)_ __ ___   __ _  __ _  ___   _ __   __ _| |_| |__
// (_|_) | '_ \ / _` | | | '_ ` _ \ / _` |/ _` |/ _ \ | '_ \ / _` | __| '_ \
//  _ _  | |_) | (_| | | | | | | | | (_| | (_| |  __/ | |_) | (_| | |_| | | |
// (_|_) |_.__/ \__, | |_|_| |_| |_|\__,_|\__, |\___| | .__/ \__,_|\__|_| |_|
//              |___/                     |___/       |_|

fn cli_bg_image_path(c: &mut Config,
                     a: &mut Args) -> Result<(), Box<dyn Error>>
{
    c.window.bg.image =
        a.next()
        .ok_or("No image path provided")?;

    Ok(())
}


//  _ _   _ __   __ _ _ __ ___   ___
// (_|_) | '_ \ / _` | '_ ` _ \ / _ \
//  _ _  | | | | (_| | | | | | |  __/
// (_|_) |_| |_|\__,_|_| |_| |_|\___|

fn cli_name(c: &mut Config,
                  a: &mut Args) -> Result<(), Box<dyn Error>>
{
    c.model.name = a.next();

    Ok(())
}

//                    _   _
//  _ _   _ __   __ _| |_| |__
// (_|_) | '_ \ / _` | __| '_ \
//  _ _  | |_) | (_| | |_| | | |
// (_|_) | .__/ \__,_|\__|_| |_|
//       |_|

fn cli_path(c: &mut Config,
                  a: &mut Args) -> Result<(), Box<dyn Error>>
{
    c.model.path =
        a.next()
        .ok_or("No model path provided")?;

    Ok(())
}

//                        _   _
//  _ _   _ __ ___   ___ | |_(_) ___  _ __  ___    ___  _ __   ___ _ __
// (_|_) | '_ ` _ \ / _ \| __| |/ _ \| '_ \/ __|  / _ \| '_ \ / _ \ '_ \
//  _ _  | | | | | | (_) | |_| | (_) | | | \__ \ | (_) | |_) |  __/ | | |
// (_|_) |_| |_| |_|\___/ \__|_|\___/|_| |_|___/  \___/| .__/ \___|_| |_|
//                                                     |_|

fn cli_motions_open(c: &mut Config,
                          a: &mut Args) -> Result<(), Box<dyn Error>>
{
    let motions =
        a.next()
        .ok_or("No model launch motions provided")?;

    c.model.motions.open =
        motions.split(',')
        .map(|t| {
            let mut s = t.rsplit(':');
            let m =
                s.next()
                .expect("motion")
                .to_string();
            let c = match s.next() {
                Some(c) => c.to_string(),
                None    => "".to_string()
            };

            (c, m)
        })
        .collect();

    Ok(())
}

//                        _   _                   _     _ _
//  _ _   _ __ ___   ___ | |_(_) ___  _ __  ___  (_) __| | | ___
// (_|_) | '_ ` _ \ / _ \| __| |/ _ \| '_ \/ __| | |/ _` | |/ _ \
//  _ _  | | | | | | (_) | |_| | (_) | | | \__ \ | | (_| | |  __/
// (_|_) |_| |_| |_|\___/ \__|_|\___/|_| |_|___/ |_|\__,_|_|\___|

fn cli_motion_idle(c: &mut Config,
                         a: &mut Args) -> Result<(), Box<dyn Error>>
{
    let motion =
        a.next()
        .ok_or("No model idle motion provided")?;

    let mut split = motion.rsplit(':');
    let id =
        split.next()
        .expect("motion")
        .to_string();
    let class = match split.next() {
        Some(c) => c.to_string(),
        None    => "".to_string()
    };

    c.model.motions.idle = Some((class, id));

    Ok(())
}

//                        _   _                                  _
//  _ _   _ __ ___   ___ | |_(_) ___  _ __  ___   _   _ ___ _ __/ |
// (_|_) | '_ ` _ \ / _ \| __| |/ _ \| '_ \/ __| | | | / __| '__| |
//  _ _  | | | | | | (_) | |_| | (_) | | | \__ \ | |_| \__ \ |  | |
// (_|_) |_| |_| |_|\___/ \__|_|\___/|_| |_|___/  \__,_|___/_|  |_|

fn cli_motion_usr1(c: &mut Config,
                         a: &mut Args) -> Result<(), Box<dyn Error>>
{
    let motion =
        a.next()
        .ok_or("No model SIGUSR1 motion provided")?;

    let mut split = motion.rsplit(':');
    let id =
        split.next()
        .expect("motion")
        .to_string();
    let class = match split.next() {
        Some(c) => c.to_string(),
        None    => "".to_string()
    };

    c.model.motions.usr1 = Some((class, id));

    Ok(())
}

