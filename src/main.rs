use std::time::{Instant, Duration};
use glium::{
    glutin,
    Surface,
    uniform,
    texture::{SrgbTexture2d, RawImage2d},
};
use serde::{Serialize, Deserialize};

const APP_NAME: &'static str = "rusty-ships";
const TARGET_FPS:        u64 = 60;

#[derive(Serialize, Deserialize)]
struct Config {
    window: WindowConfig,
    model: ModelConfig,
}

#[derive(Serialize, Deserialize)]
struct WindowConfig {
    size: [u16; 2],
    title: String,
}

#[derive(Serialize, Deserialize)]
struct ModelConfig {
    name: Option<String>,
    path: String,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            window: WindowConfig {
                size: [800, 600],
                title: "Rusty Ships".to_string(),
            },
            model: ModelConfig {
                name: None,
                path: "assets".to_string(),
            },
        }
    }
}

fn main() -> Result<(), String> {

    let config: Config = 
        confy::load(APP_NAME, None)
        .map_err(|e| format!("Failed to load config: {e}"))?;
    let path =
        confy::get_configuration_file_path(APP_NAME, None)
        .map_err(|e| format!("Error getting assets path: {e}"))
        .and_then(|mut conf| {conf.pop(); Ok(conf)})
        .and_then(|path| Ok(path.join(config.model.path)))?;
    let model_name =
        config.model.name
        .ok_or(format!("No model provided"))?;

    //   ____ _       _       _ _
    //  / ___| |     (_)_ __ (_) |_
    // | |  _| |     | | '_ \| | __|
    // | |_| | |___  | | | | | | |_
    //  \____|_____| |_|_| |_|_|\__|

    let event_loop = glutin::event_loop::EventLoop::new();
    let display = {
        use glutin::{
            window::WindowBuilder,
            dpi::LogicalSize,
            ContextBuilder,
        };

        let (width, height) = config.window.size.into();
        let title = config.window.title;
        glium::Display::new(WindowBuilder::new()
                            .with_inner_size(LogicalSize::new(width,
                                                              height))
                            .with_title(title)
                            .with_decorations(false)
                            .with_transparent(true),
                            ContextBuilder::new(),
                            &event_loop)
        .map_err(|e| format!("Failed to create display: {e}"))
    }?;

    let program = {
        use glium::program::Program;

        Program::from_source(&display,
                             include_str!("vert.glsl"),
                             include_str!("frag.glsl"),
                             None)
        .map_err(|e| format!("Failed to build shaders: {e}"))
    }?;

    ////                      _      _
    ////  _ __ ___   ___   __| | ___| |
    //// | '_ ` _ \ / _ \ / _` |/ _ \ |
    //// | | | | | | (_) | (_| |  __/ |
    //// |_| |_| |_|\___/ \__,_|\___|_|

    //let mut model = {
    //    let result = framework::Model::new(&path,
    //                                       &model_name);
    //    match result {
    //        Ok(m)  => m,
    //        Err(e) => die("Failed to load model", e)
    //    }
    //};
    //info("Loaded model");

    //let canvas = model.l2d.canvas_info();

    //let textures: Vec<SrgbTexture2d> =
    //    model.textures.iter()
    //    .map(|image| {
    //        let image_dimensions = image.dimensions();
    //        let image_raw =
    //            RawImage2d::from_raw_rgba_reversed(&image.clone().into_raw(),
    //                                               image_dimensions);
    //        let texture = SrgbTexture2d::new(&display, image_raw);
    //        match texture {
    //            Ok(t)  => t,
    //            Err(e) => die("Failed to load texture", e)
    //        }
    //    }).collect();
    //info("Loaded textures");

    //                       _     _
    //   _____   _____ _ __ | |_  | | ___   ___  _ __
    //  / _ \ \ / / _ \ '_ \| __| | |/ _ \ / _ \| '_ \
    // |  __/\ V /  __/ | | | |_  | | (_) | (_) | |_) |
    //  \___| \_/ \___|_| |_|\__| |_|\___/ \___/| .__/
    //                                          |_|

    let inc = 1000 / TARGET_FPS;
    let mut limiter = Instant::now();

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

        limiter += Duration::from_millis(inc);
        control_flow.set_wait_until(limiter);
        //model.update();
        //let parts = model.parts_sorted();

        //let buffers: Vec<_> = {
        //    use glium::{
        //        vertex::VertexBuffer,
        //        index::{IndexBuffer, PrimitiveType},
        //    };

        //    parts.iter()
        //    .map(|part| {
        //        let vbuffer = VertexBuffer::new(&display,
        //                                        &part.vertices);
        //        let v = match vbuffer {
        //            Ok(v)  => v,
        //            Err(e) => die("Failed to create vertex buffer", e)
        //        };

        //        let ibuffer = IndexBuffer::new(&display,
        //                                      PrimitiveType::TrianglesList,
        //                                      &part.indices);
        //        let i = match ibuffer {
        //            Ok(i)  => i,
        //            Err(e) => die("Failed to create index buffer", e)
        //        };

        //        (v, i)
        //    }).collect()
        //};

        let mut frame = display.draw();
        frame.clear_color(0.,
                          0.,
                          0.,
                          0.);

        //for i in 0..parts.len() {
        //    let uniforms = uniform!{
        //        size: canvas.size_in_pixels,
        //        origin: canvas.origin_in_pixels,
        //        scale: canvas.pixels_per_unit,
        //        opacity: model.opacity * parts[i].opacity,
        //        tex: &textures[parts[i].texture_index],
        //    };

        //    if !parts[i].visibility {continue}

        //    let params = &glium::DrawParameters {
        //        blend: parts[i].blend,
        //        .. Default::default()
        //    };

        //    frame
        //    .draw(&buffers[i].0,
        //          &buffers[i].1,
        //          &program,
        //          &uniforms,
        //          &params)
        //    .unwrap_or_else(|e| die("Failed to draw", e));
        //}

        frame
        .finish()
        .unwrap_or_else(|e| eprintln!("Failed to create frame: {e}"));

        match event {
            Event::WindowEvent {event, ..} => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                _ => {}
            }
            //Event::DeviceEvent {event, ..} => match event {
            //    DeviceEvent::Key(KeyboardInput {
            //        virtual_keycode: Some(vkc),
            //        state: ElementState::Pressed,
            //        ..
            //    }) => {
            //        let id = match vkc {VKC::Key0 | VKC::Numpad0 => Some(0),
            //                            VKC::Key1 | VKC::Numpad1 => Some(1),
            //                            VKC::Key2 | VKC::Numpad2 => Some(2),
            //                            VKC::Key3 | VKC::Numpad3 => Some(3),
            //                            VKC::Key4 | VKC::Numpad4 => Some(4),
            //                            VKC::Key5 | VKC::Numpad5 => Some(5),
            //                            VKC::Key6 | VKC::Numpad6 => Some(6),
            //                            VKC::Key7 | VKC::Numpad7 => Some(7),
            //                            VKC::Key8 | VKC::Numpad8 => Some(8),
            //                            VKC::Key9 | VKC::Numpad9 => Some(9),
            //                            VKC::A                   => Some(10),
            //                            VKC::B                   => Some(11),
            //                            VKC::C                   => Some(12),
            //                            VKC::D                   => Some(13),
            //                            VKC::E                   => Some(14),
            //                            VKC::F                   => Some(15),
            //                            _                        => None};
            //        if let Some(id2) = id {
            //            let result = model.set_motion(id2);
            //            info(&format!("Set motion to {}", result));
            //        }
            //    }
            //    _ => {}
            //}
            _ => {}
        }
    });
}

