use std::{
    fs,
    env,
    thread,
    rc::Rc,
    io::Read,
    path::Path,
    time::Instant,
    cmp::{max, min},
    os::unix::net::UnixListener,
};
use glium::{
    Frame,
    Display,
    Surface,
    program::Program,
    backend::{Facade, Context},
    glutin::{
        ContextBuilder,
        dpi::LogicalSize,
        window::WindowBuilder,
        event::{Event, WindowEvent},
        event_loop::EventLoopBuilder,
    },
};
use signal_hook::{
    iterator::Signals,
    consts::{SIGTERM, SIGUSR1},
};

mod config;
use config::{Config, FitConfig, constant::*};

mod message;
use message::{Message, SOCKET_ADDR};

mod framework;
use framework::Model;

mod xsecurelock;
use xsecurelock::XSecureLock;

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

    //                        _     _
    //    _____   _____ _ __ | |_  | | ___   ___  _ __
    //   / _ \ \ / / _ \ '_ \| __| | |/ _ \ / _ \| '_ \
    //  |  __/\ V /  __/ | | | |_  | | (_) | (_) | |_) |
    // (_)___| \_/ \___|_| |_|\__| |_|\___/ \___/| .__/
    //                                           |_|

    let event_loop =
        EventLoopBuilder::<Message>::with_user_event()
        .build();

    let path = Path::new(SOCKET_ADDR);

    if path.exists() {
        eprintln!("Removing existing socket before connecting");
        match fs::remove_file(path) {
            Ok(r)  => r,
            Err(e) => eprintln!("Unable to remove existing socket: {e}")
        }
    }

    let listener =
        UnixListener::bind(path)
        .map_err(|e| format!("Failed to listen to the socket: {e}"))?;

    let proxy = event_loop.create_proxy();

    thread::spawn(move || {
        listener.incoming()
        .for_each(|stream| {
            stream
            .map_err(|e| format!("Socket connection error: {e}"))
            .map(|mut s| {
                let mut input = String::new();
                s.read_to_string(&mut input).unwrap_or(0);
                input
            })
            .and_then(|i| Message::parse(i)
                          .ok_or(format!("Failed to parse message")))
            .and_then(|m| proxy.send_event(m)
                          .map_err(|e| format!("Failed to send message: {e}")))
            .unwrap_or_else(|e| eprintln!("{e}"))
        });
    });

    let mut signals =
        Signals::new(&[SIGTERM, SIGUSR1])
        .map_err(|e| format!("Failed to listen to signals: {e}"))?;

    let proxy = event_loop.create_proxy();
    let usr1 = config.model.motions.usr1.clone();

    thread::spawn(move || {
        signals.forever()
        .for_each(|signal| {
            match signal {
                SIGTERM => proxy.send_event(Message::Exit),
                SIGUSR1 => match usr1.clone() {
                    Some(t) => proxy.send_event(Message::SetMotion(t)),
                    None    => Ok(())
                }
                _       => Ok(())
            }.unwrap_or(());
        });
    });

    //       _ _           _
    //    __| (_)___ _ __ | | __ _ _   _
    //   / _` | / __| '_ \| |/ _` | | | |
    //  | (_| | \__ \ |_) | | (_| | |_| |
    // (_)__,_|_|___/ .__/|_|\__,_|\__, |
    //              |_|            |___/

    let display = match env::var("XSCREENSAVER_WINDOW") {
        Ok(xwin) => {
            let xwin: u64 =
                xwin.parse()
                .expect("xsecurelock to provide valid window id");
            Hack::XSecureLock(XSecureLock::new(xwin)?)
        }
        Err(_) => {
            let (width, height) = config.window.size.into();
            let title = config.window.title.clone();
            let display =
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
                .map_err(|e| format!("Failed to create display: {e}"))?;
            Hack::Display(display)
        }
    };

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
        let (width, height) =
            display
            .get_context()
            .get_framebuffer_dimensions();

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
            Event::NewEvents(_) => {
                let elapsed =
                    last_frame
                    .elapsed()
                    .as_secs_f64();
                last_frame = Instant::now();

                model
                .update(elapsed)
                .unwrap_or_else(|e| eprintln!("Failed to update model: {e}"));
            }
            Event::UserEvent(msg) => match msg {
                    Message::SetMotion(m) => model.queue((m.0.as_str(),
                                                          m.1.as_str())),
                    Message::Toggle       => model.toggle(),
                    Message::Pause        => model.pause(),
                    Message::Play         => model.play(),
                    Message::Exit         => Some(control_flow.set_exit()),
                }.unwrap_or(()),
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
            Event::RedrawEventsCleared => control_flow.set_poll(),
            Event::LoopDestroyed => eprintln!("Good bye!"),
            _ => {}
        }
    });
}

enum Hack {
    XSecureLock(XSecureLock),
    Display(Display),
}

impl Hack {
    fn draw(&self) -> Frame {
        match self {
            Hack::XSecureLock(x) => x.draw(),
            Hack::Display(d)     => d.draw(),
        }
    }
}

impl Facade for Hack {
    fn get_context(&self) -> &Rc<Context> {
        match self {
            Hack::XSecureLock(x) => x.get_context(),
            Hack::Display(d)     => d.get_context(),
        }
    }
}
