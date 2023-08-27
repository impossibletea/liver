use std::{
    io::Read,
    path::Path,
    fs::{self, File},
};
use image;
use glium::{
    glutin,
    Display,
    Surface,
    implement_vertex, uniform,
    texture::SrgbTexture2d,
};
use live2d_cubism_core_sys::core as l2d;

const INIT_WIDTH:          u16 = 640;
const INIT_HEIGHT:         u16 = 480;
const MODEL_NAME: &'static str = "qianwei_2";

struct Part {
    vertices: Vec<Vert>,
    indices: Vec<u16>,
    opacity: f32,
    order: i32,
    visibility: bool,
    screen_color: [f32; 4],
    multiply_color: [f32; 4],
    texture_index: usize,
    masks: Vec<usize>,
}

#[derive(Copy, Clone, Debug)]
struct Vert {
    position: [f32; 2],
    texture_uv: [f32; 2],
}

implement_vertex!(Vert,
                  position,
                  texture_uv);

fn create_parts(model: &l2d::Model) -> Vec<Part> {
    // STATIC PARTS
    let texture_uvs_set: Vec<Vec<[f32; 2]>> =
        model.drawables
        .iter()
        .map(|drawable| drawable.vertex_uvs
                        .iter()
                        .map(|uv| [uv.x, uv.y])
                        .collect())
        .collect();
    let texture_indices_set: Vec<usize> =
        model.drawables
        .iter()
        .map(|drawable| drawable.texture_index)
        .collect();
    let masks_set: Vec<Vec<usize>> =
        model.drawables
        .iter()
        .map(|drawable| drawable.masks.to_vec())
        .collect();
    let triangle_indices_set: Vec<Vec<u16>> =
        model.drawables
        .iter()
        .map(|drawable| drawable.triangle_indices.to_vec())
        .collect();

    // DYNAMIC PARTS
    let dynamic = model.dynamic.read();

    let positions_set = dynamic.drawable_vertex_position_containers();
    let opacities_set = dynamic.drawable_opacities();
    let orders_set = dynamic.drawable_render_orders();
    let screen_colors_set = dynamic.drawable_screen_colors();
    let multiply_colors_set = dynamic.drawable_multiply_colors();

    let mut result: Vec<_> =
        (0..positions_set.len())
        .into_iter()
        .map(|part| {
            let vertices =
                (0..positions_set[part].len())
                .into_iter()
                .map(|vertex| {
                    let p = positions_set[part][vertex];
                    Vert {
                        position: [p.x, p.y],
                        texture_uv: texture_uvs_set[part][vertex],
                    }
                })
                .collect();
            let indices = triangle_indices_set[part].clone();
            let texture_index = texture_indices_set[part];
            let masks = masks_set[part].clone();
            let sc = screen_colors_set[part];
            let mc = multiply_colors_set[part];

            Part {
                vertices,
                indices,
                masks,
                texture_index,
                opacity: opacities_set[part],
                order: orders_set[part],
                visibility: true,
                screen_color: [sc.x, sc.y, sc.z, sc.w],
                multiply_color: [mc.x, mc.y, mc.z, mc.w],
            }
        })
        .collect();

    result.sort_by_key(|a| a.order);
    result
}

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let display = {
        use glutin::{
            window::WindowBuilder,
            dpi::LogicalSize,
            ContextBuilder,
        };

        glium::Display::new(WindowBuilder::new()
                            .with_inner_size(LogicalSize::new(INIT_WIDTH,
                                                              INIT_HEIGHT))
                            .with_title("Rusty Ships")
                            .with_decorations(false)
                            .with_transparent(true),
                            ContextBuilder::new(),
                            &event_loop)
        .unwrap()
    };

    let program = {
        use glium::program::Program;

        let shader_path = Path::new("./src/");
        let (vert_shader_src, frag_shader_src) =
            (fs::read_to_string(shader_path.join("vert.glsl")).unwrap(),
             fs::read_to_string(shader_path.join("frag.glsl")).unwrap());

        Program::from_source(&display,
                             &vert_shader_src,
                             &frag_shader_src,
                             None).unwrap()
    };

    let params = {
        use glium::{DrawParameters, draw_parameters::Blend};

        DrawParameters {
        blend: Blend::alpha_blending(),
        .. Default::default()
        }
    };

    let model_path = Path::new("./res/assets").join(MODEL_NAME);
    let model = load_model(&model_path,
                           MODEL_NAME);

    let canvas = model.canvas_info;

    let textures = {
        let mut indices: Vec<usize> =
            model.drawables
            .iter()
            .map(|drawable| drawable.texture_index)
            .collect();
        indices.sort();
        indices.dedup();

        load_textures(&model_path.join("textures"),
                      indices,
                      &display)
    };

    let mut dynamic = model.dynamic.write();
    dynamic.reset_drawable_dynamic_flags();
    dynamic.update();
    drop(dynamic);

    let parts = create_parts(&model);

    let vertex_buffers: Vec<_> = {
        use glium::vertex::VertexBuffer;

        parts.iter()
        .map(|part| VertexBuffer::new(&display,
                                      &part.vertices).unwrap())
        .collect()
    };

    let index_buffers: Vec<_> = {
        use glium::index::{IndexBuffer, PrimitiveType};
    
        parts.iter()
        .map(|part| IndexBuffer::new(&display,
                                     PrimitiveType::TrianglesList,
                                     &part.indices).unwrap())
        .collect()
    };

    event_loop.run(move |event,
                         _,
                         control_flow| {
        use glutin::event::{Event, WindowEvent};

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
                opacity: parts[i].opacity,
                tex: &textures[parts[i].texture_index],
            };

            if !parts[i].visibility {continue}

            frame.draw(&vertex_buffers[i],
                       &index_buffers[i],
                       &program,
                       &uniforms,
                       &params).unwrap();
        }

        //(0..vertex_buffers.len())
        //.into_iter()
        //.for_each(|i| frame.draw(&vertex_buffers[i],
        //              &index_buffers[i],
        //              &program,
        //              &uniforms,
        //              &params).unwrap());
        frame.finish().unwrap();

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                _ => {},
            },
            _ => {}
        }
    });
}

fn load_model(root: &Path,
              name: &str) -> l2d::Model {
    use l2d::{CubismCore, Model};

    let model_name = name.to_string() + ".moc3";
    let mut model_file = File::open(root.join(model_name)).unwrap();
    let mut model_bytes = Vec::new();
    model_file.read_to_end(&mut model_bytes).unwrap();

    let model_moc =
        CubismCore::default().moc_from_bytes(&model_bytes).unwrap();

    Model::from_moc(&model_moc)
}

fn load_textures(root: &Path,
                 indices: Vec<usize>,
                 display: &Display) -> Vec<SrgbTexture2d> {
    use std::io::BufReader;
    use glium::texture::RawImage2d;

    indices
    .iter()
    .map(|index| {
        let texture_id = format!("texture_{:02}.png", index);
        let image_file = File::open(root.join(texture_id)).unwrap();
        let image =
            image::load(BufReader::new(image_file),
                        image::ImageFormat::Png).unwrap()
            .to_rgba8();
        let image_dimensions = image.dimensions();
        let image_raw = RawImage2d::from_raw_rgba_reversed(&image.into_raw(),
                                                           image_dimensions);
        SrgbTexture2d::new(display, image_raw).unwrap()
    })
    .collect()
}

