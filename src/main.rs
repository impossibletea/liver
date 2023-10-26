use std::time::{Instant, Duration};
use glium::{glutin, Surface};
use serde::{Serialize, Deserialize};

mod framework;
use framework::Model;

const APP_NAME:   &'static str = "rusty-ships";
const CONFIG:     &'static str = "config";
const TARGET_FPS: u64          = 60;

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
    pub size:  [u16; 2],
    pub title: String,
}

#[derive(Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: Option<String>,
    pub path: String,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            window: WindowConfig {
                size:  [800, 600],
                title: "Rusty Ships".to_string(),
            },
            model: ModelConfig {
                name: None,
                path: "assets".to_string(),
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

    let event_loop = glutin::event_loop::EventLoop::new();
    let display = {
        use glutin::{
            window::WindowBuilder,
            dpi::LogicalSize,
            ContextBuilder,
            platform::unix::{WindowBuilderExtUnix, XWindowType},
        };
        use glium::Display;

        let (width, height) = config.window.size.into();
        let title = config.window.title.clone();
        let window_type = vec![XWindowType::Desktop];

        Display::new(WindowBuilder::new()
                     .with_inner_size(LogicalSize::new(width, height))
                     .with_title(title)
                     .with_decorations(false)
                     .with_transparent(true),
                     //.with_x11_window_type(window_type),
                     ContextBuilder::new(),
                     &event_loop)
        .map_err(|e| format!("Failed to create display: {e}"))
    }?;


    //    _ __  _ __ ___   __ _ _ __ __ _ _ __ ___
    //   | '_ \| '__/ _ \ / _` | '__/ _` | '_ ` _ \
    //  _| |_) | | | (_) | (_| | | | (_| | | | | | |
    // (_) .__/|_|  \___/ \__, |_|  \__,_|_| |_| |_|
    //   |_|              |___/

    let program = {
        use glium::program::Program;

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

    let inc = 1000 / TARGET_FPS;
    let mut last_frame = Instant::now();
    let mut limiter = Instant::now();

    model.play();

    event_loop.run(move |event,
                         _,
                         control_flow| {
        use glutin::event::{
            Event,
            WindowEvent,
            DeviceEvent,
            KeyboardInput,
            ElementState,
            VirtualKeyCode as VKC,
        };

        let elapsed =
            last_frame
            .elapsed()
            .as_secs_f64();
        last_frame = Instant::now();

        model
        .update(elapsed,
                &display)
        .unwrap_or_else(|e| eprintln!("Failed to update model: {e}"));

        let mut frame = display.draw();
        frame.clear_color(0.,
                          0.,
                          0.,
                          0.);

        model
        .draw(&mut frame,
              &program)
        .unwrap_or_else(|e| eprintln!("Failed to draw model: {e}"));

        frame
        .finish()
        .unwrap_or_else(|e| eprintln!("Failed to create frame: {e}"));

        match event {
            Event::WindowEvent {event, ..} => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                _ => {}
            }
            Event::DeviceEvent {event, ..} => match event {
                DeviceEvent::Key(KeyboardInput {
                    virtual_keycode: Some(vkc),
                    state: ElementState::Pressed,
                    ..
                }) => {
                    let id = match vkc {VKC::Key0 | VKC::Numpad0 => Some(0),
                                        VKC::Key1 | VKC::Numpad1 => Some(1),
                                        VKC::Key2 | VKC::Numpad2 => Some(2),
                                        VKC::Key3 | VKC::Numpad3 => Some(3),
                                        VKC::Key4 | VKC::Numpad4 => Some(4),
                                        VKC::Key5 | VKC::Numpad5 => Some(5),
                                        VKC::Key6 | VKC::Numpad6 => Some(6),
                                        VKC::Key7 | VKC::Numpad7 => Some(7),
                                        VKC::Key8 | VKC::Numpad8 => Some(8),
                                        VKC::Key9 | VKC::Numpad9 => Some(9),
                                        VKC::A                   => Some(10),
                                        VKC::B                   => Some(11),
                                        VKC::C                   => Some(12),
                                        VKC::D                   => Some(13),
                                        VKC::E                   => Some(14),
                                        VKC::F                   => Some(15),
                                        _                        => None};
                    if let Some(id2) = id {
                        let result =
                            model
                            .set_motion(id2)
                            .ok_or_else(|| eprintln!("No motion {id2}"));
                    }
                }
                _ => {}
            }
            _ => {}
        }

        limiter += Duration::from_millis(inc);
        control_flow.set_wait_until(limiter);
    });
}

