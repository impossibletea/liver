use std::default::Default;
use serde::{Serialize, Deserialize};

pub mod constant {
    pub const APP_NAME:   &'static str = "rusty-ships";
    pub const CONFIG:     &'static str = "config";
}

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

#[derive(Serialize, Deserialize)]
pub struct WindowConfig {
    pub size:  [u32; 2],
    pub title: String,
    pub fit:   FitConfig,
}

#[derive(Serialize, Deserialize)]
pub enum FitConfig {
    Contain,
    Cover,
}

#[derive(Serialize, Deserialize)]
pub struct ModelConfig {
    pub name:    Option<String>,
    pub path:    String,
    pub motions: MotionConfig,
}

#[derive(Serialize, Deserialize)]
pub struct MotionConfig {
    pub open: Option<Vec<String>>,
    pub idle: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window: WindowConfig {
                size:  [800, 600],
                title: "Rusty Ships".to_string(),
                fit:   FitConfig::Cover,
            },
            model: ModelConfig {
                name:    None,
                path:    "assets".to_string(),
                motions: MotionConfig {
                    open: None,
                    idle: None,
                },
            },
        }
    }
}

