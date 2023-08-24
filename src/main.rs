use std::{
    time::Instant,
    path::Path,
    fs::{self, File},
    io::Read,
};
use glium::{
    glutin,
    Surface,
    implement_vertex,
    uniform,
};
use live2d_cubism_core_sys::core as l2d;

const  INIT_WIDTH:    u16 = 640;
const  INIT_HEIGHT:   u16 = 480;
static WINDOW_TITLE: &str = "Rusty Ships";
static MODEL_PATH:   &str = "./res/assets/z23/";

fn main() {
    let shader_path = Path::new("./src/");
    let vert_shader_src =
        fs::read_to_string(shader_path.join("vert.glsl"))
        .unwrap();
    let frag_shader_src =
        fs::read_to_string(shader_path.join("frag.glsl"))
        .unwrap();

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb =
        glutin::window::WindowBuilder::new()
        .with_inner_size(glutin::dpi::LogicalSize::new(INIT_WIDTH,
                                                       INIT_HEIGHT))
        .with_title(WINDOW_TITLE)
        .with_decorations(false)
        .with_transparent(true);
    let cb = glutin::ContextBuilder::new();
    let display =
        glium::Display::new(wb,
                            cb,
                            &event_loop)
        .unwrap();

    let now = Instant::now();

    let black: (f32, f32, f32) =
        [0x2D, 0x2C, 0x2F]
        .map(|x| x as f32 / 0xFF as f32)
        .into();

    let program =
        glium::program::Program::from_source(&display,
                                             &vert_shader_src,
                                             &frag_shader_src,
                                             None)
        .unwrap();

    let params = glium::DrawParameters::default();

    let model_path = Path::new(MODEL_PATH);
    let mut model_file = 
        File::open(model_path.join("z23.moc3"))
        .unwrap();
    let mut model_bytes = Vec::new();
    model_file
    .read_to_end(&mut model_bytes)
    .unwrap();

    let model_moc =
        l2d::CubismCore::default()
        .moc_from_bytes(&model_bytes)
        .unwrap();

    let model = l2d::Model::from_moc(&model_moc);

    let mut dynamic = model.dynamic.write();
    dynamic.reset_drawable_dynamic_flags();
    dynamic.update();

    let vertices: Vec<Vec<Vert>> =
        dynamic
        .drawable_vertex_position_containers()
        .iter()
        .map(|triangle| triangle
                        .iter()
                        .map(|vertex| Vert{ position: [vertex.x, vertex.y]})
                        .collect())
        .collect();
    let vertex_buffer =
        glium::vertex::VertexBuffer::new(&display,
                                         &vertices[0])
        .unwrap();

    let indices: Vec<Vec<u16>> =
        model
        .drawables
        .iter()
        .map(|drawable| drawable
                        .triangle_indices
                        .to_vec())
        .collect();

    let index_buffer =
        glium::index::IndexBuffer::new(&display,
                                       glium::index::PrimitiveType::TrianglesList,
                                       &indices[0])
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        use glutin::event::{Event, WindowEvent};

        let time =
            Instant::now()
            .duration_since(now)
            .as_secs_f32();

        let uniforms = uniform!{
            time: time,
        };

        let mut frame = display.draw();
        frame.clear_color(0.,
                          0.,
                          0.,
                          0.);

        frame
        .draw(&vertex_buffer,
              &index_buffer,
              &program,
              &uniforms,
              &params)
        .unwrap();
        frame
        .finish()
        .unwrap();

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                _ => {},
            },
            _ => {}
        }
    });
}

#[derive(Copy, Clone, Debug)]
struct Vert {
    position: [f32; 2],
}

implement_vertex!(Vert, position);

