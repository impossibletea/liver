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

const INIT_WIDTH:            u16 = 1000;
const INIT_HEIGHT:           u16 = 1000;
const WINDOW_TITLE: &'static str = "Rusty Ships";
const MODEL_NAME:   &'static str = "kelaimengsuo_2";

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
            Err(e) => die("Failed to create display", e)
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
            Err(e) => die("Failed to build shaders", e)
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
            Err(e) => die("Failed to load model", e)
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
                Err(e) => die("Failed to load texture", e)
            }
        }).collect();
    info("Loaded textures");

    //                       _     _
    //   _____   _____ _ __ | |_  | | ___   ___  _ __
    //  / _ \ \ / / _ \ '_ \| __| | |/ _ \ / _ \| '_ \
    // |  __/\ V /  __/ | | | |_  | | (_) | (_) | |_) |
    //  \___| \_/ \___|_| |_|\__| |_|\___/ \___/| .__/
    //                                          |_|

    let mut time: f32 = 0.;

    event_loop.run(move |event,
                         _,
                         control_flow| {
        use glutin::event::{Event, WindowEvent};

        model.update(time);
        let parts = model.parts_sorted();

        let buffers: Vec<_> = {
            use glium::{
                vertex::VertexBuffer,
                index::{IndexBuffer, PrimitiveType},
            };

            parts.iter()
            .map(|part| {
                let vbuffer = VertexBuffer::new(&display,
                                                &part.vertices);
                let v = match vbuffer {
                    Ok(v)  => v,
                    Err(e) => die("Failed to create vertex buffer", e)
                };

                let ibuffer = IndexBuffer::new(&display,
                                              PrimitiveType::TrianglesList,
                                              &part.indices);
                let i = match ibuffer {
                    Ok(i)  => i,
                    Err(e) => die("Failed to create index buffer", e)
                };

                (v, i)
            }).collect()
        };

        let mut frame = display.draw();
        frame.clear_color(0.,
                          0.,
                          0.,
                          0.);

        for i in 0..parts.len() {
            let uniforms = uniform!{
                size: canvas.size_in_pixels,
                origin: canvas.origin_in_pixels,
                scale: canvas.pixels_per_unit,
                opacity: model.opacity * parts[i].opacity,
                tex: &textures[parts[i].texture_index],
            };

            if !parts[i].visibility {continue}

            let params = &glium::DrawParameters {
                blend: parts[i].blend,
                .. Default::default()
            };

            frame
            .draw(&buffers[i].0,
                  &buffers[i].1,
                  &program,
                  &uniforms,
                  &params)
            .unwrap_or_else(|e| die("Failed to draw", e));
        }

        frame
        .finish()
        .unwrap_or_else(|e| err("Failed to create frame", e));

        time += 0.005;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                _ => {},
            },
            _ => {}
        }
    });
}

