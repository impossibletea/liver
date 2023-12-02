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
    pub bg:    BgConfig,
}

#[derive(Serialize, Deserialize)]
pub enum FitConfig {
    Contain,
    Cover,
}

#[derive(Serialize, Deserialize)]
pub struct BgConfig {
    pub variant: BgType,
    pub color:   [f32; 4],
    pub image:   String,
}

#[derive(Serialize, Deserialize)]
pub enum BgType {
    Color,
    Image,
}

#[derive(Serialize, Deserialize)]
pub struct ModelConfig {
    pub name:    Vec<String>,
    pub path:    String,
    pub motions: MotionConfig,
}

#[derive(Serialize, Deserialize)]
pub struct MotionConfig {
    pub open: Vec<(String, String)>,
    pub idle: Option<(String, String)>,
    pub usr1: Option<(String, String)>,
}

impl Default for Config {
    fn default() -> Self
    {
        Self {
            window: WindowConfig {
                size:  [800, 600],
                title: "Rusty Ships".to_string(),
                fit:   FitConfig::Cover,
                bg:    BgConfig {
                    variant: BgType::Color,
                    color:   [0., 0., 0., 0.],
                    image:   "".to_string(),
                },
            },
            model: ModelConfig {
                name:    Vec::new(),
                path:    "assets".to_string(),
                motions: MotionConfig {
                    open: Vec::new(),
                    idle: None,
                    usr1: None,
                },
            },
        }
    }
}

