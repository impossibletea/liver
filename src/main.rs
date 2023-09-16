use std::path::Path;
use glium::{
    glutin,
    Surface,
    uniform,
    texture::{SrgbTexture2d, RawImage2d},
};

mod framework;
mod logging;
use logging::*;

const INIT_WIDTH:            u16 = 640;
const INIT_HEIGHT:           u16 = 480;
const WINDOW_TITLE: &'static str = "Rusty Ships";
const MODEL_NAME:   &'static str = "wuerlixi_2";

fn main() {

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

        let result =
            glium::Display::new(WindowBuilder::new()
                                .with_inner_size(LogicalSize::new(INIT_WIDTH,
                                                                  INIT_HEIGHT))
                                .with_title(WINDOW_TITLE)
                                .with_decorations(false)
                                .with_transparent(true),
                                ContextBuilder::new(),
                                &event_loop);

        match result {
            Ok(d)  => d,
            Err(e) => shit_yourself_and_die("Failed to create display", e)
        }
    };
    info("Created a display");

    let program = {
        use glium::program::Program;

        let program =
            Program::from_source(&display,
                                 include_str!("vert.glsl"),
                                 include_str!("frag.glsl"),
                                 None);
        match program {
            Ok(p)  => p,
            Err(e) => shit_yourself_and_die("Failed to build shaders", e)
        }
    };
    info("Loaded shaders");

    //                      _      _ 
    //  _ __ ___   ___   __| | ___| |
    // | '_ ` _ \ / _ \ / _` |/ _ \ |
    // | | | | | | (_) | (_| |  __/ |
    // |_| |_| |_|\___/ \__,_|\___|_|

    let path = Path::new("./res/assets");
    let mut model = {
        let result = framework::Model::new(&path,
                                           MODEL_NAME);
        match result {
            Ok(m)  => m,
            Err(e) => shit_yourself_and_die("Failed to load model", e)
        }
    };
    info("Loaded model");

    let canvas = model.l2d.canvas_info();

    let textures: Vec<SrgbTexture2d> =
        model.textures.iter()
        .map(|image| {
            let image_dimensions = image.dimensions();
            let image_raw =
                RawImage2d::from_raw_rgba_reversed(&image.clone().into_raw(),
                                                   image_dimensions);
            let texture = SrgbTexture2d::new(&display, image_raw);
            match texture {
                Ok(t)  => t,
                Err(e) => shit_yourself_and_die("Failed to load texture", e)
            }
        }).collect();

    let vertex_buffers: Vec<_> = {
        use glium::vertex::VertexBuffer;

        model.parts.iter()
        .map(|part| VertexBuffer::new(&display,
                                      &part.vertices).unwrap())
        .collect()
    };

    let index_buffers: Vec<_> = {
        use glium::index::{IndexBuffer, PrimitiveType};

        model.parts.iter()
        .map(|part| IndexBuffer::new(&display,
                                     PrimitiveType::TrianglesList,
                                     &part.indices).unwrap())
        .collect()
    };
    info("Loaded textures");

    //                       _     _                   
    //   _____   _____ _ __ | |_  | | ___   ___  _ __  
    //  / _ \ \ / / _ \ '_ \| __| | |/ _ \ / _ \| '_ \ 
    // |  __/\ V /  __/ | | | |_  | | (_) | (_) | |_) |
    //  \___| \_/ \___|_| |_|\__| |_|\___/ \___/| .__/ 
    //                                          |_|    

    event_loop.run(move |event,
                         _,
                         control_flow| {
        use glutin::event::{Event, WindowEvent};

        let mut frame = display.draw();
        frame.clear_color(0.,
                          0.,
                          0.,
                          0.);

        for i in 0..model.parts.len() {
            let uniforms = uniform!{
                size: canvas.size_in_pixels,
                origin: canvas.origin_in_pixels,
                scale: canvas.pixels_per_unit,
                opacity: model.parts[i].opacity,
                tex: &textures[model.parts[i].texture_index],
            };

            if !model.parts[i].visibility {continue}

            let params = &glium::DrawParameters {
                blend: model.parts[i].blend,
                .. Default::default()
            };

            frame.draw(&vertex_buffers[i],
                       &index_buffers[i],
                       &program,
                       &uniforms,
                       &params).unwrap();
            .unwrap_or_else(|e| shit_yourself_and_die("Failed to draw", e));
        }

        frame
        .finish()
        .unwrap_or_else(|e| err("Failed to create frame", e));

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                _ => {},
            },
            _ => {}
        }
    });
}

