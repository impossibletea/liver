use std::{
    fs,
    env,
    thread,
    rc::Rc,
    io::Read,
    path::Path,
    error::Error,
    time::Instant,
    os::unix::net::UnixListener,
};
use glium::{
    Frame,
    Display,
    Surface,
    uniform,
    program::Program,
    vertex::VertexBuffer,
    backend::{Facade, Context},
    index::{IndexBuffer, PrimitiveType},
    texture::{RawImage2d, SrgbTexture2d},
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
use config::{Config, FitConfig, BgType, constant::*};

mod message;
use message::{Message, SOCKET_ADDR};

mod framework;
use framework::{Model, Vert};

mod xsecurelock;
use xsecurelock::XSecureLock;

//                  _
//  _ __ ___   __ _(_)_ __
// | '_ ` _ \ / _` | | '_ \
// | | | | | | (_| | | | | |
// |_| |_| |_|\__,_|_|_| |_|

fn main() -> Result<(), Box<dyn Error>>
{

    //                    __ _
    //    ___ ___  _ __  / _(_) __ _
    //   / __/ _ \| '_ \| |_| |/ _` |
    //  | (_| (_) | | | |  _| | (_| |
    // (_)___\___/|_| |_|_| |_|\__, |
    //                         |___/

    let config: Config = confy::load(APP_NAME,
                                     CONFIG)?;

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

    let listener = UnixListener::bind(path)?;

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

    let mut signals = Signals::new(&[SIGTERM, SIGUSR1])?;

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
                             &event_loop)?;
            Hack::Display(display)
        }
    };

    //    _ __  _ __ ___   __ _ _ __ __ _ _ __ ___
    //   | '_ \| '__/ _ \ / _` | '__/ _` | '_ ` _ \
    //  _| |_) | | | (_) | (_| | | | (_| | | | | | |
    // (_) .__/|_|  \___/ \__, |_|  \__,_|_| |_| |_|
    //   |_|              |___/

    let program = Program::from_source(&display,
                                       include_str!("vert.glsl"),
                                       include_str!("frag.glsl"),
                                       None)?;

    let bg_program = Program::from_source(&display,
                                          include_str!("bg.glsl"),
                                          include_str!("frag.glsl"),
                                          None)?;

    //    _                _                                   _
    //   | |__   __ _  ___| | ____ _ _ __ ___  _   _ _ __   __| |
    //   | '_ \ / _` |/ __| |/ / _` | '__/ _ \| | | | '_ \ / _` |
    //  _| |_) | (_| | (__|   < (_| | | | (_) | |_| | | | | (_| |
    // (_)_.__/ \__,_|\___|_|\_\__, |_|  \___/ \__,_|_| |_|\__,_|
    //                         |___/

    let background_image = match &config.window.bg.variant {
        BgType::Image => {
            let bg =
                config.window.bg.image.as_ref()
                .ok_or(Box::from("No image provided"))
                .and_then(|bg| Background::new(bg, &display))?;
            Some(bg)
        }
        BgType::Color => None,
    };

    //                        _      _
    //    _ __ ___   ___   __| | ___| |
    //   | '_ ` _ \ / _ \ / _` |/ _ \ |
    //  _| | | | | | (_) | (_| |  __/ |
    // (_)_| |_| |_|\___/ \__,_|\___|_|

    let screen_index: usize =
        env::var("XSCREENSAVER_SAVER_INDEX")
        .unwrap_or("0".to_string())
        .parse()
        .unwrap_or(0);

    let mut model = Model::new(&config,
                               screen_index,
                               &display)?;

    //  _ _   _ __ _   _ _ __
    // (_|_) | '__| | | | '_ \
    //  _ _  | |  | |_| | | | |
    // (_|_) |_|   \__,_|_| |_|

    let mut last_frame = Instant::now();

    let mut aspect = {
        let frame =
            display
            .get_context()
            .get_framebuffer_dimensions();
        let view = [
            frame.0 as f32,
            frame.1 as f32
        ];
        let object = model.size();
        let fit = &config.window.fit;

        calc_aspect(object,
                    view,
                    fit)
    };

    let mut bg_aspect = {
        let frame =
            display
            .get_context()
            .get_framebuffer_dimensions();
        let view = [
            frame.0 as f32,
            frame.1 as f32
        ];
        let object = match &background_image {
            Some(bg) => bg.size,
            None     => [1., 1.]
        };
        let fit = &FitConfig::Cover;

        calc_aspect(object,
                    view,
                    fit)
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
                        let view = [
                            s.width as f32,
                            s.height as f32
                        ];
                        let object = model.size();
                        let fit = &config.window.fit;

                        calc_aspect(object,
                                    view,
                                    fit)
                    };
                    bg_aspect = {
                        let view = [
                            s.width as f32,
                            s.height as f32
                        ];
                        let object = match &background_image {
                            Some(bg) => bg.size,
                            None     => [1., 1.]
                        };
                        let fit = &FitConfig::Cover;

                        calc_aspect(object,
                                    view,
                                    fit)
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

                match &config.window.bg.variant {
                    BgType::Image => {
                        let bg =
                            background_image.as_ref()
                            .expect("image to be here");
                        let op: f32 = 1.;
                        let uniforms = uniform!{
                            tex: &bg.texture,
                            aspect: bg_aspect,
                            opacity: op,
                        };

                        frame.draw(&bg.vertex_buffer,
                                   &bg.index_buffer,
                                   &bg_program,
                                   &uniforms,
                                   &Default::default())
                        .unwrap_or_else(|e| eprintln!("{e}"))
                    }
                    BgType::Color => {
                        let c =
                            config.window.bg.color.as_ref()
                            .unwrap_or(&[0., 0., 0., 0.]);
                        frame.clear_color(c[0], c[1], c[2], c[3])
                    }
                }

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

//  _   _            _
// | | | | __ _  ___| | __
// | |_| |/ _` |/ __| |/ /
// |  _  | (_| | (__|   <
// |_| |_|\__,_|\___|_|\_\

enum Hack {
    XSecureLock(XSecureLock),
    Display(Display),
}

impl Hack {
    fn draw(&self) -> Frame
    {
        match self {
            Hack::XSecureLock(x) => x.draw(),
            Hack::Display(d)     => d.draw(),
        }
    }
}

impl Facade for Hack {
    fn get_context(&self) -> &Rc<Context>
    {
        match self {
            Hack::XSecureLock(x) => x.get_context(),
            Hack::Display(d)     => d.get_context(),
        }
    }
}

//  ____             _                                   _
// | __ )  __ _  ___| | ____ _ _ __ ___  _   _ _ __   __| |
// |  _ \ / _` |/ __| |/ / _` | '__/ _ \| | | | '_ \ / _` |
// | |_) | (_| | (__|   < (_| | | | (_) | |_| | | | | (_| |
// |____/ \__,_|\___|_|\_\__, |_|  \___/ \__,_|_| |_|\__,_|
//                       |___/

struct Background {
    texture:       SrgbTexture2d,
    vertex_buffer: VertexBuffer<Vert>,
    index_buffer:  IndexBuffer<u16>,
    size:          [f32; 2],
}

impl Background {
    fn new<T>(bg: &str,
              display: &T) -> Result<Self, Box<dyn Error>>
    where T: Facade
    {
        let bg_path =
            confy::get_configuration_file_path(APP_NAME,
                                               CONFIG)
            .map(|mut conf| {
                conf.pop();
                conf.join(&bg)
            })?;

        let image =
            image::open(&bg_path)?
            .to_rgba8();
        let image_dimensions = image.dimensions();
        let (x, y) = (image_dimensions.0 as f32,
                      image_dimensions.1 as f32);
        let size = [x, y];
        let (rx, ry) = (x/y, 1.);
        let image_raw =
            RawImage2d::from_raw_rgba_reversed(&image.into_raw(),
                                               image_dimensions);

        let texture = SrgbTexture2d::new(display,
                                         image_raw)?;

        let vertices = vec![
            Vert::new([-rx, -ry], [0., 0.]),
            Vert::new([-rx,  ry], [0., 1.]),
            Vert::new([ rx, -ry], [1., 0.]),
            Vert::new([ rx,  ry], [1., 1.]),
        ];
        let indices: Vec<u16> = vec![0, 1, 2, 3];

        let vertex_buffer = VertexBuffer::new(display,
                                              &vertices)?;
        let index_buffer = IndexBuffer::new(display,
                                            PrimitiveType::TriangleStrip,
                                            &indices)?;

        Ok(Self {
            texture,
            vertex_buffer,
            index_buffer,
            size,
        })
    }
}

//                                  _
//  _ _    __ _ ___ _ __   ___  ___| |_
// (_|_)  / _` / __| '_ \ / _ \/ __| __|
//  _ _  | (_| \__ \ |_) |  __/ (__| |_
// (_|_)  \__,_|___/ .__/ \___|\___|\__|
//                 |_|

fn calc_aspect(object: [f32; 2],
               view:   [f32; 2],
               fit:    &FitConfig) -> [f32; 2]
{
    let view_ratio = view[0]/view[1];
    let object_ratio = object[0]/object[1];
    let plus_w = [1./view_ratio, 1.];
    let plus_h = [1./object_ratio, view_ratio/object_ratio];
    let mode = object_ratio < view_ratio;

    match fit {
        FitConfig::Contain => if mode {plus_w} else {plus_h}
        FitConfig::Cover   => if mode {plus_h} else {plus_w}
    }
}
