use std::{
    iter::zip,
    path::PathBuf,
};
use glium::{
    DrawParameters,
    Blend,
    BlendingFunction,
    LinearBlendingFactor as F,
    Surface,
    uniform,
    implement_vertex,
    program::Program,
    vertex::VertexBuffer,
    index::{IndexBuffer, PrimitiveType},
    texture::{SrgbTexture2d, RawImage2d},
};
use cubism::{
    model::UserModel,
    json::model::Model3,
};

//  __  __           _      _
// |  \/  | ___   __| | ___| |
// | |\/| |/ _ \ / _` |/ _ \ |
// | |  | | (_) | (_| |  __/ |
// |_|  |_|\___/ \__,_|\___|_|

pub struct Model {
    name:      String,
    path:      PathBuf,
    json:      Model3,
    moc:       UserModel,
    canvas:    CanvasInfo,
    textures:  Vec<SrgbTexture2d>,
    drawables: Vec<Drawable>,
}

struct Drawable {
    vertex_buffer: VertexBuffer<Vert>,
    index_buffer:  IndexBuffer<u16>,
    opacity:       f32,
    texture_index: i32,
    render_order:  i32,
}

struct CanvasInfo {
    size:   [f32; 2],
    origin: [f32; 2],
    scale:  f32,
}

impl Model {

    //  _ _   _ __   _____      __
    // (_|_) | '_ \ / _ \ \ /\ / /
    //  _ _  | | | |  __/\ V  V /
    // (_|_) |_| |_|\___| \_/\_/

    pub fn new(config:  &crate::Config,
               display: &glium::Display) -> Result<Self, String> {

        //    _ __   __ _ _ __ ___   ___
        //   | '_ \ / _` | '_ ` _ \ / _ \
        //  _| | | | (_| | | | | | |  __/
        // (_)_| |_|\__,_|_| |_| |_|\___|

        let name =
            config.model.name.clone()
            .ok_or(format!("No model provided"))?;

        //                _   _
        //    _ __   __ _| |_| |__
        //   | '_ \ / _` | __| '_ \
        //  _| |_) | (_| | |_| | | |
        // (_) .__/ \__,_|\__|_| |_|
        //   |_|

        let path =
            confy::get_configuration_file_path(crate::APP_NAME,
                                               crate::CONFIG)
            .map_err(|e| format!("Error getting assets path: {e}"))
            .and_then(|mut conf| {conf.pop(); Ok(conf)})?
            .join(&config.model.path)
            .join(&name);

        //     _
        //    (_)___  ___  _ __
        //    | / __|/ _ \| '_ \
        //  _ | \__ \ (_) | | | |
        // (_)/ |___/\___/|_| |_|
        //  |__/

        let json =
            std::fs::File::open(path.join(format!("{name}.model3.json")))
            .map_err(|e| format!("Error opening json: {e}"))
            .and_then(|f| Model3::from_reader(f)
                          .map_err(|e| format!("Error parsing json: {e}")))?;

        //    _ __ ___   ___   ___
        //   | '_ ` _ \ / _ \ / __|
        //  _| | | | | | (_) | (__
        // (_)_| |_| |_|\___/ \___|

        let mut moc =
            UserModel::from_model3(&path,
                                   &json)
            .map_err(|e| format!("Error creating model: {e}"))?;

        //    ___ __ _ _ ____   ____ _ ___
        //   / __/ _` | '_ \ \ / / _` / __|
        //  | (_| (_| | | | \ V / (_| \__ \
        // (_)___\__,_|_| |_|\_/ \__,_|___/

        let canvas = {
            let t = moc.canvas_info();

            CanvasInfo {
                size:   t.0,
                origin: t.1,
                scale:  t.2,
            }
        };

        //   _            _
        //  | |_ _____  _| |_ _   _ _ __ ___  ___
        //  | __/ _ \ \/ / __| | | | '__/ _ \/ __|
        //  | ||  __/>  <| |_| |_| | | |  __/\__ \
        // (_)__\___/_/\_\\__|\__,_|_|  \___||___/

        let textures: Vec<_> =
            Result::from_iter(
                json.file_references.textures.iter()
                .map(|r| {
                    let t_path = path.join(&r);
                    let image =
                        image::open(&t_path)
                        .map_err(|e| format!("Error opening texture: {e}"))?
                        .to_rgba8();
                    let image_dimensions = image.dimensions();
                    let image_raw =
                        RawImage2d::from_raw_rgba_reversed(&image.into_raw(),
                                                           image_dimensions);
                    SrgbTexture2d::new(display,
                                       image_raw)
                    .map_err(|e| format!("Error creating texture: {e}"))
                })
            )?;

        //       _                         _     _
        //    __| |_ __ __ ___      ____ _| |__ | | ___  ___
        //   / _` | '__/ _` \ \ /\ / / _` | '_ \| |/ _ \/ __|
        //  | (_| | | | (_| |\ V  V / (_| | |_) | |  __/\__ \
        // (_)__,_|_|  \__,_| \_/\_/ \__,_|_.__/|_|\___||___/

        let mut drawables: Vec<_> =
            Result::from_iter(
                moc.drawables()
                .map(|d| Drawable::new(d, display))
            )?;
        drawables.sort_unstable_by_key(|d| d.render_order);

        //           _
        //  _ __ ___| |_ _   _ _ __ _ __
        // | '__/ _ \ __| | | | '__| '_ \
        // | | |  __/ |_| |_| | |  | | | |
        // |_|  \___|\__|\__,_|_|  |_| |_|

        Ok(Self {
            name,
            path,
            json,
            moc,
            canvas,
            textures,
            drawables,
        })
    }

    //            _
    //  _ _    __| |_ __ __ ___      __
    // (_|_)  / _` | '__/ _` \ \ /\ / /
    //  _ _  | (_| | | | (_| |\ V  V /
    // (_|_)  \__,_|_|  \__,_| \_/\_/

    pub fn draw<T: Surface>(&self,
                            frame:   &mut T,
                            program: &Program) -> Result<(), String> {
        Result::from(
            self.drawables.iter()
            .map(|d| {
                let uniforms = uniform!{
                    size:    self.canvas.size,
                    origin:  self.canvas.origin,
                    scale:   self.canvas.scale,
                    opacity: d.opacity,
                    tex:     &self.textures[d.texture_index as usize],
                };

                let params = &DrawParameters {
                    blend: Blend {
                        color: BlendingFunction::Addition {
                            source: F::SourceAlpha,
                            destination: F::OneMinusSourceAlpha,
                        },
                        alpha: BlendingFunction::Addition {
                            source: F::One,
                            destination: F::OneMinusSourceAlpha,
                        },
                        .. Default::default()
                    },
                    .. Default::default()
                };

                frame.draw(&d.vertex_buffer,
                           &d.index_buffer,
                           program,
                           &uniforms,
                           &params)
                .map_err(|e| format!("Failed to draw: {e}"))
            }).collect()
        )
    }
}

impl Drawable {

    //  _ _   _ __   _____      __
    // (_|_) | '_ \ / _ \ \ /\ / /
    //  _ _  | | | |  __/\ V  V /
    // (_|_) |_| |_|\___| \_/\_/

    fn new(drawable: cubism::core::Drawable,
           display:  &glium::Display) -> Result<Self, String> {

        //                 _              _            __  __
        // __   _____ _ __| |_ _____  __ | |__  _   _ / _|/ _| ___ _ __
        // \ \ / / _ \ '__| __/ _ \ \/ / | '_ \| | | | |_| |_ / _ \ '__|
        //  \ V /  __/ |  | ||  __/>  <  | |_) | |_| |  _|  _|  __/ |
        // (_)_/ \___|_|   \__\___/_/\_\ |_.__/ \__,_|_| |_|  \___|_|

        let vertices: Vec<_> =
            zip(drawable.vertex_positions,
                drawable.vertex_uvs)
            .map(|(pos, uv)| Vert{
                position:   *pos,
                texture_uv: *uv,
            }).collect();
        let vertex_buffer =
            VertexBuffer::new(display,
                              &vertices)
            .map_err(|e| format!("Failed to create vertex buffer: {e}"))?;

        //    _           _             _            __  __
        //   (_)_ __   __| | _____  __ | |__  _   _ / _|/ _| ___ _ __
        //   | | '_ \ / _` |/ _ \ \/ / | '_ \| | | | |_| |_ / _ \ '__|
        //  _| | | | | (_| |  __/>  <  | |_) | |_| |  _|  _|  __/ |
        // (_)_|_| |_|\__,_|\___/_/\_\ |_.__/ \__,_|_| |_|  \___|_|

        let index_buffer =
            IndexBuffer::new(display,
                             PrimitiveType::TrianglesList,
                             drawable.indices)
            .map_err(|e| format!("Failed to create index buffer: {e}"))?;

        //                          _ _
        //    ___  _ __   __ _  ___(_) |_ _   _
        //   / _ \| '_ \ / _` |/ __| | __| | | |
        //  | (_) | |_) | (_| | (__| | |_| |_| |
        // (_)___/| .__/ \__,_|\___|_|\__|\__, |
        //        |_|                     |___/

        let opacity = drawable.opacity;

        //   _            _                    _           _
        //  | |_ _____  _| |_ _   _ _ __ ___  (_)_ __   __| | _____  __
        //  | __/ _ \ \/ / __| | | | '__/ _ \ | | '_ \ / _` |/ _ \ \/ /
        //  | ||  __/>  <| |_| |_| | | |  __/ | | | | | (_| |  __/>  <
        // (_)__\___/_/\_\\__|\__,_|_|  \___| |_|_| |_|\__,_|\___/_/\_\

        let texture_index = drawable.texture_index;

        //                       _                           _
        //    _ __ ___ _ __   __| | ___ _ __    ___  _ __ __| | ___ _ __
        //   | '__/ _ \ '_ \ / _` |/ _ \ '__|  / _ \| '__/ _` |/ _ \ '__|
        //  _| | |  __/ | | | (_| |  __/ |    | (_) | | | (_| |  __/ |
        // (_)_|  \___|_| |_|\__,_|\___|_|     \___/|_|  \__,_|\___|_|

        let render_order = drawable.render_order;

        //           _
        //  _ __ ___| |_ _   _ _ __ _ __
        // | '__/ _ \ __| | | | '__| '_ \
        // | | |  __/ |_| |_| | |  | | | |
        // |_|  \___|\__|\__,_|_|  |_| |_|

        Ok(Drawable {
            vertex_buffer,
            index_buffer,
            opacity,
            texture_index,
            render_order,
        })
    }

    //                        _       _
    //  _ _   _   _ _ __   __| | __ _| |_ ___
    // (_|_) | | | | '_ \ / _` |/ _` | __/ _ \
    //  _ _  | |_| | |_) | (_| | (_| | ||  __/
    // (_|_)  \__,_| .__/ \__,_|\__,_|\__\___|
    //             |_|

    fn update(&mut self) {}
}

// __     __        _
// \ \   / /__ _ __| |_ _____  __
//  \ \ / / _ \ '__| __/ _ \ \/ /
//   \ V /  __/ |  | ||  __/>  <
//    \_/ \___|_|   \__\___/_/\_\

#[derive(Copy, Clone)]
struct Vert {
    position:   [f32; 2],
    texture_uv: [f32; 2],
}

implement_vertex!(Vert, position, texture_uv);

