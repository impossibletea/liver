use std::error::Error;

use serde::{Serialize, Deserialize};

pub mod constant {
    pub const APP_NAME:   &'static str = "liver";
    pub const CONFIG:     &'static str = "config";
}
use constant::*;

mod cli;

//   ____             __ _
//  / ___|___  _ __  / _(_) __ _
// | |   / _ \| '_ \| |_| |/ _` |
// | |__| (_) | | | |  _| | (_| |
//  \____\___/|_| |_|_| |_|\__, |
//                         |___/

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub window: WindowConfig,
    pub model:  ModelConfig,
}

// __        ___           _                ____             __ _
// \ \      / (_)_ __   __| | _____      __/ ___|___  _ __  / _(_) __ _
//  \ \ /\ / /| | '_ \ / _` |/ _ \ \ /\ / / |   / _ \| '_ \| |_| |/ _` |
//   \ V  V / | | | | | (_| | (_) \ V  V /| |__| (_) | | | |  _| | (_| |
//    \_/\_/  |_|_| |_|\__,_|\___/ \_/\_/  \____\___/|_| |_|_| |_|\__, |
//                                                                |___/

#[derive(Serialize, Deserialize)]
pub struct WindowConfig {
    pub size:  [u32; 2],
    pub title: String,
    pub fit:   FitConfig,
    pub bg:    BgConfig,
}

//  _____ _ _    ____             __ _
// |  ___(_) |_ / ___|___  _ __  / _(_) __ _
// | |_  | | __| |   / _ \| '_ \| |_| |/ _` |
// |  _| | | |_| |__| (_) | | | |  _| | (_| |
// |_|   |_|\__|\____\___/|_| |_|_| |_|\__, |
//                                     |___/

#[derive(Serialize, Deserialize)]
pub enum FitConfig {
    Contain,
    Cover,
}

//  ____         ____             __ _
// | __ )  __ _ / ___|___  _ __  / _(_) __ _
// |  _ \ / _` | |   / _ \| '_ \| |_| |/ _` |
// | |_) | (_| | |__| (_) | | | |  _| | (_| |
// |____/ \__, |\____\___/|_| |_|_| |_|\__, |
//        |___/                        |___/

#[derive(Serialize, Deserialize)]
pub struct BgConfig {
    pub variant: BgType,
    pub color:   [f32; 4],
    pub image:   String,
}

//  ____       _____
// | __ )  __ |_   _|   _ _ __   ___
// |  _ \ / _` || || | | | '_ \ / _ \
// | |_) | (_| || || |_| | |_) |  __/
// |____/ \__, ||_| \__, | .__/ \___|
//        |___/     |___/|_|

#[derive(Serialize, Deserialize)]
pub enum BgType {
    Color,
    Image,
}

//  __  __           _      _  ____             __ _
// |  \/  | ___   __| | ___| |/ ___|___  _ __  / _(_) __ _
// | |\/| |/ _ \ / _` |/ _ \ | |   / _ \| '_ \| |_| |/ _` |
// | |  | | (_) | (_| |  __/ | |__| (_) | | | |  _| | (_| |
// |_|  |_|\___/ \__,_|\___|_|\____\___/|_| |_|_| |_|\__, |
//                                                   |___/

#[derive(Serialize, Deserialize)]
pub struct ModelConfig {
    pub name:    Option<String>,
    pub path:    String,
    pub motions: MotionConfig,
}

//  __  __       _   _              ____             __ _
// |  \/  | ___ | |_(_) ___  _ __  / ___|___  _ __  / _(_) __ _
// | |\/| |/ _ \| __| |/ _ \| '_ \| |   / _ \| '_ \| |_| |/ _` |
// | |  | | (_) | |_| | (_) | | | | |__| (_) | | | |  _| | (_| |
// |_|  |_|\___/ \__|_|\___/|_| |_|\____\___/|_| |_|_| |_|\__, |
//                                                        |___/

#[derive(Serialize, Deserialize)]
pub struct MotionConfig {
    pub open: Vec<(String, String)>,
    pub idle: Option<(String, String)>,
}

//   ____             __ _
//  / ___|___  _ __  / _(_) __ _   _ _
// | |   / _ \| '_ \| |_| |/ _` | (_|_)
// | |__| (_) | | | |  _| | (_| |  _ _
//  \____\___/|_| |_|_| |_|\__, | (_|_)
//                         |___/

impl Config {
    pub fn new() -> Result<Config, Box<dyn Error>>
    {
        let mut config: Config = confy::load(APP_NAME, CONFIG)?;
        let _program = cli::cli_args(&mut config)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self
    {
        Self {
            window: WindowConfig {
                size:  [800, 600],
                title: "Liver".to_string(),
                fit:   FitConfig::Cover,
                bg:    BgConfig {
                    variant: BgType::Color,
                    color:   [0., 0., 0., 0.],
                    image:   "".to_string(),
                },
            },
            model: ModelConfig {
                name:    None,
                path:    "assets".to_string(),
                motions: MotionConfig {
                    open: Vec::new(),
                    idle: None,
                },
            },
        }
    }
}

