use std::{
    time::Instant,
    cmp::{max, min},
};
use glium::{
    Display,
    Surface,
    glutin::{
        ContextBuilder,
        dpi::LogicalSize,
        event_loop::EventLoop,
        window::WindowBuilder,
        event::{Event, WindowEvent},
    },
    program::Program,
};
use serde::{Serialize, Deserialize};

mod framework;
use framework::Model;

const APP_NAME:   &'static str = "rusty-ships";
const CONFIG:     &'static str = "config";

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

impl std::default::Default for Config {
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

//                  _
//  _ __ ___   __ _(_)_ __
// | '_ ` _ \ / _` | | '_ \
// | | | | | | (_| | | | | |
// |_| |_| |_|\__,_|_|_| |_|

fn main() -> Result<(), String> {

    //                    __ _
    //    ___ ___  _ __  / _(_) __ _
    //   / __/ _ \| '_ \| |_| |/ _` |
    //  | (_| (_) | | | |  _| | (_| |
    // (_)___\___/|_| |_|_| |_|\__, |
    //                         |___/

    let config: Config =
        confy::load(APP_NAME,
                    CONFIG)
        .map_err(|e| format!("Failed to load config: {e}"))?;

    //       _ _           _
    //    __| (_)___ _ __ | | __ _ _   _
    //   / _` | / __| '_ \| |/ _` | | | |
    //  | (_| | \__ \ |_) | | (_| | |_| |
    // (_)__,_|_|___/ .__/|_|\__,_|\__, |
    //              |_|            |___/

    let (width, height) = config.window.size.into();
    let event_loop = EventLoop::new();
    let display = {
        let title = config.window.title.clone();

        Display::new(WindowBuilder::new()
                     .with_inner_size(LogicalSize::new(width,
                                                       height))
                     .with_title(title)
                     .with_decorations(false)
                     .with_transparent(true),
                     ContextBuilder::new()
                     .with_vsync(true)
                     .with_double_buffer(Some(true)),
                     &event_loop)
        .map_err(|e| format!("Failed to create display: {e}"))
    }?;

    //    _ __  _ __ ___   __ _ _ __ __ _ _ __ ___
    //   | '_ \| '__/ _ \ / _` | '__/ _` | '_ ` _ \
    //  _| |_) | | | (_) | (_| | | | (_| | | | | | |
    // (_) .__/|_|  \___/ \__, |_|  \__,_|_| |_| |_|
    //   |_|              |___/

    let program = {
        Program::from_source(&display,
                             include_str!("vert.glsl"),
                             include_str!("frag.glsl"),
                             None)
        .map_err(|e| format!("Failed to build shaders: {e}"))
    }?;

    //                        _      _
    //    _ __ ___   ___   __| | ___| |
    //   | '_ ` _ \ / _ \ / _` |/ _ \ |
    //  _| | | | | | (_) | (_| |  __/ |
    // (_)_| |_| |_|\___/ \__,_|\___|_|

    let mut model = Model::new(&config,
                               &display)?;

    //  _ _   _ __ _   _ _ __
    // (_|_) | '__| | | | '_ \
    //  _ _  | |  | |_| | | | |
    // (_|_) |_|   \__,_|_| |_|

    let mut last_frame = Instant::now();

    let mut aspect = {
        let r = max(width, height) as f32 / min(width, height) as f32;
        let (x, y) = match config.window.fit {
            FitConfig::Contain => (1./r, 1.),
            FitConfig::Cover   => (1.,    r),
        };
        if width > height {[x, y]} else {[y, x]}
    };

    event_loop.run(move |event,
                         _,
                         control_flow| {
        match event {
            Event::WindowEvent {event, ..} => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::Resized(s) => {
                    aspect = {
                        let (w, h) = (s.width, s.height);
                        let r = max(w, h) as f32 / min(w, h) as f32;
                        let (x, y) = match config.window.fit {
                            FitConfig::Contain => (1./r, 1.),
                            FitConfig::Cover   => (1.,    r),
                        };
                        if w > h {[x, y]} else {[y, x]}
                    };
                }
                _ => {}
            }
            Event::MainEventsCleared => {
                let elapsed =
                    last_frame
                    .elapsed()
                    .as_secs_f64();
                last_frame = Instant::now();

                model
                .update(elapsed)
                .unwrap_or_else(|e| eprintln!("Failed to update model: {e}"));

                //      _                          _             _
                //   __| |_ __ __ ___      __  ___| |_ __ _ _ __| |_
                //  / _` | '__/ _` \ \ /\ / / / __| __/ _` | '__| __|
                // | (_| | | | (_| |\ V  V /  \__ \ || (_| | |  | |_
                //  \__,_|_|  \__,_| \_/\_/   |___/\__\__,_|_|   \__|

                let mut frame = display.draw();
                frame.clear_color(0.,
                                  0.,
                                  0.,
                                  0.);

                model
                .draw(&mut frame,
                      &program,
                      aspect)
                .unwrap_or_else(|e| eprintln!("Failed to draw model: {e}"));

                frame
                .finish()
                .unwrap_or_else(|e| eprintln!("Failed to create frame: {e}"));

                //      _                       __ _       _     _
                //   __| |_ __ __ ___      __  / _(_)_ __ (_)___| |__
                //  / _` | '__/ _` \ \ /\ / / | |_| | '_ \| / __| '_ \
                // | (_| | | | (_| |\ V  V /  |  _| | | | | \__ \ | | |
                //  \__,_|_|  \__,_| \_/\_/   |_| |_|_| |_|_|___/_| |_|

            }
            Event::RedrawEventsCleared => {
                control_flow.set_poll();
            }
            _ => {}
        }
    });
}

