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

mod config;
use config::{Config, FitConfig, BgType};

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
    use ProgramVariant as PV;

    //                    __ _
    //    ___ ___  _ __  / _(_) __ _
    //   / __/ _ \| '_ \| |_| |/ _` |
    //  | (_| (_) | | | |  _| | (_| |
    // (_)___\___/|_| |_|_| |_|\__, |
    //                         |___/

    let config = Config::new()?;

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
        fs::remove_file(path)?
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
                          .ok_or("Failed to parse message".to_string()))
            .and_then(|m| proxy.send_event(m)
                          .map_err(|e| format!("Failed to send message: {e}")))
            .unwrap_or_else(|e| eprintln!("{e}"))
        });
    });

    //       _ _           _
    //    __| (_)___ _ __ | | __ _ _   _
    //   / _` | / __| '_ \| |/ _` | | | |
    //  | (_| | \__ \ |_) | | (_| | |_| |
    // (_)__,_|_|___/ .__/|_|\__,_|\__, |
    //              |_|            |___/

    let display = if let Ok(xwin) = env::var("XSCREENSAVER_WINDOW") {
        let xwin: u64 =
            xwin.parse()
            .expect("xsecurelock to provide valid window id");
        Hack::XSecureLock(XSecureLock::new(xwin)?)
    } else {
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
    };

    //    _ __  _ __ ___   __ _ _ __ __ _ _ __ ___  ___
    //   | '_ \| '__/ _ \ / _` | '__/ _` | '_ ` _ \/ __|
    //  _| |_) | | | (_) | (_| | | | (_| | | | | | \__ \
    // (_) .__/|_|  \___/ \__, |_|  \__,_|_| |_| |_|___/
    //   |_|              |___/

    let prg_background =
        Program::from_source(&display,
                             include_str!("shaders/bg_vert.glsl"),
                             include_str!("shaders/bg_frag.glsl"),
                             None)?;

    let prg_mask =
        Program::from_source(&display,
                             include_str!("shaders/mask_vert.glsl"),
                             include_str!("shaders/mask_frag.glsl"),
                             None)?;

    let prg_normal =
        Program::from_source(&display,
                             include_str!("shaders/vert.glsl"),
                             include_str!("shaders/frag.glsl"),
                             None)?;

    // Ideally I'd like to initialize array with Rc::new_zeroed() and then fill
    // with correct indices using PV::NormalBlend etc., but this method for Rc
    // is currently in nightly, so I just have to make sure that correct
    // programs are at correct indices
    let programs: [Rc<Program>; PV::Counter as usize] = [
        Rc::new(prg_background),
        Rc::new(prg_mask),
        Rc::new(prg_normal),
    ];

    //    _                _                                   _
    //   | |__   __ _  ___| | ____ _ _ __ ___  _   _ _ __   __| |
    //   | '_ \ / _` |/ __| |/ / _` | '__/ _ \| | | | '_ \ / _` |
    //  _| |_) | (_| | (__|   < (_| | | | (_) | |_| | | | | (_| |
    // (_)_.__/ \__,_|\___|_|\_\__, |_|  \___/ \__,_|_| |_|\__,_|
    //                         |___/

    let background_image = match &config.window.bg.variant {
        BgType::Image => Some(Background::new(config.window.bg.image.as_str(),
                                              &display)?),
        BgType::Color => None,
    };

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
        let object =
            background_image.as_ref()
            .map(|bg| bg.size)
            .unwrap_or([1., 1.]);
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
                    Message::SetMotion(m)   => model.set((m.0.as_str(),
                                                          m.1.as_str())),
                    Message::QueueMotion(m) => model.queue((m.0.as_str(),
                                                            m.1.as_str())),
                    Message::Toggle         => model.toggle(),
                    Message::Pause          => model.pause(),
                    Message::Play           => model.play(),
                    #[allow(clippy::unit_arg)]
                    Message::Exit           => Some(control_flow.set_exit()),
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
                        let object =
                            background_image.as_ref()
                            .map(|bg| bg.size)
                            .unwrap_or([1., 1.]);
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
                        let uniforms = uniform!{
                            tex: &bg.texture,
                            aspect: bg_aspect,
                        };

                        frame.draw(&bg.vertex_buffer,
                                   &bg.index_buffer,
                                   &programs[PV::Background as usize],
                                   &uniforms,
                                   &Default::default())
                        .unwrap_or_else(|e| eprintln!("{e}"))
                    }
                    BgType::Color => {
                        let c = config.window.bg.color;
                        frame.clear_color(c[0], c[1], c[2], c[3])
                    }
                }

                model
                .draw(&mut frame,
                      &programs,
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

//  ____
// |  _ \ _ __ ___   __ _ _ __ __ _ _ __ ___  ___
// | |_) | '__/ _ \ / _` | '__/ _` | '_ ` _ \/ __|
// |  __/| | | (_) | (_| | | | (_| | | | | | \__ \
// |_|   |_|  \___/ \__, |_|  \__,_|_| |_| |_|___/
//                  |___/

#[derive(Copy, Clone)]
enum ProgramVariant {
    Background = 0,
    Mask,
    BlendNormal,

    Counter
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
        let bg_path = expanduser::expanduser(bg)?;

        let image =
            image::open(bg_path)?
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
