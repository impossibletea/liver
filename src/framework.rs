use std::{
    fs::File,
    path::Path,
    io::{Read, BufReader},
};
use glium::{
    implement_vertex,
    draw_parameters::{
        Blend,
        BlendingFunction,
        LinearBlendingFactor as Factor,
    },
};
use live2d_cubism_core_sys::core as l2d;
use l2d::{
    CubismCore,
    Model as L2DModel,
};

mod motion;
mod framework_json;
use framework_json::JsonModel;

pub struct Part {
    pub vertices: Vec<Vert>,
    pub indices: Vec<u16>,
    pub opacity: f32,
    pub order: i32,
    pub visibility: bool,
    pub screen_color: [f32; 4],
    pub multiply_color: [f32; 4],
    pub texture_index: usize,
    pub masks: Vec<usize>,
    pub blend: glium::draw_parameters::Blend,
}

#[derive(Copy, Clone, Debug)]
pub struct Vert {
    pub position: [f32; 2],
    pub texture_uv: [f32; 2],
}

implement_vertex!(Vert,
                  position,
                  texture_uv);

pub struct Model {
    pub l2d: L2DModel,
    pub parts: Vec<Part>,
    pub textures: Vec<image::RgbaImage>,
    pub motions: Vec<motion::Motion>,
}

impl Model {
    //  _ __   _____      __
    // | '_ \ / _ \ \ /\ / /
    // | | | |  __/\ V  V /
    // |_| |_|\___| \_/\_/

    pub fn new(path: &Path,
               name: &str) -> Self {

        //    _                 
        //   (_)___  ___  _ __  
        //   | / __|/ _ \| '_ \ 
        //   | \__ \ (_) | | | |
        //  _/ |___/\___/|_| |_|
        // |__/                 

        let base_path = path.join(name);
        let json_path =
            base_path.join(name.to_string() + ".model3.json");
        let json =
            JsonModel::new(&json_path);

        let moc_path = base_path.join(json.FileReferences.Moc);
        let texture_paths: Vec<_> =
            json.FileReferences.Textures.iter()
            .map(|texture_path| base_path.join(texture_path)).collect();

        let motions_paths: Vec<_> =
            json.FileReferences.Motions[""].as_array().unwrap().iter()
            .map(|motion| {
                 let motion_path = motion["File"].as_str().unwrap();
                 base_path.join(motion_path)
            }).collect();

        //  _ _           ____     _
        // | (_)_   _____|___ \ __| |
        // | | \ \ / / _ \ __) / _` |
        // | | |\ V /  __// __/ (_| |
        // |_|_| \_/ \___|_____\__,_|

        let mut model_file = File::open(moc_path).unwrap();
        let mut model_bytes = Vec::new();
        model_file.read_to_end(&mut model_bytes).unwrap();
        let model_moc =
            CubismCore::default()
            .moc_from_bytes(&model_bytes).unwrap();

        let l2d = L2DModel::from_moc(&model_moc);

        //                   _
        //  _ __   __ _ _ __| |_ ___
        // | '_ \ / _` | '__| __/ __|
        // | |_) | (_| | |  | |_\__ \
        // | .__/ \__,_|_|   \__|___/
        // |_|

        // Static Parts
        let texture_uvs_set: Vec<Vec<[f32; 2]>> =
            l2d.drawables().iter()
            .map(|drawable| drawable.vertex_uvs().iter()
                            .map(|uv| [uv.x, uv.y]).collect()).collect();
        let texture_indices_set: Vec<usize> =
            l2d.drawables().iter()
            .map(|drawable| drawable.texture_index()).collect();
        let masks_set: Vec<Vec<usize>> =
            l2d.drawables().iter()
            .map(|drawable| drawable.masks().to_vec()).collect();
        let triangle_indices_set: Vec<Vec<u16>> =
            l2d.drawables().iter()
            .map(|drawable| drawable.triangle_indices().to_vec()).collect();
        let constant_flags_set: Vec<_> =
            l2d.drawables().iter()
            .map(|drawable| drawable.constant_flagset()).collect();

        let constant_value = (0., 0., 0., 0.,);

        let normal_blend = {
            let normal_fn = BlendingFunction::Addition {
                source: Factor::SourceAlpha,
                destination: Factor::OneMinusSourceAlpha,
            };
            let normal_alpha = BlendingFunction::Addition {
                source: Factor::One,
                destination: Factor::One,
            };
            Blend {
                color: normal_fn,
                alpha: normal_alpha,
                constant_value,
            }
        };

        let add_blend = {
            let add_fn = BlendingFunction::Addition {
                source: Factor::One,
                destination: Factor::One,
            };
            Blend {
                color: add_fn,
                alpha: add_fn,
                constant_value,
            }
        };

        let multi_blend = {
            let multi_fn = BlendingFunction::Addition {
                source: Factor::DestinationColor,
                destination: Factor::OneMinusSourceAlpha,
            };
            let multi_alpha = BlendingFunction::Addition {
                source: Factor::Zero,
                destination: Factor::One,
            };
                Blend {
                color: multi_fn,
                alpha: multi_alpha,
                constant_value,
            }
        };

        // DYNAMIC PARTS
        let dynamic = l2d.read_dynamic();

        let positions_set = dynamic.drawable_vertex_position_containers();
        let opacities_set = dynamic.drawable_opacities();
        let orders_set = dynamic.drawable_render_orders();
        let screen_colors_set = dynamic.drawable_screen_colors();
        let multiply_colors_set = dynamic.drawable_multiply_colors();

        let mut parts: Vec<Part> =
            (0..positions_set.len()).into_iter()
            .map(|part| {
                let vertices =
                    (0..positions_set[part].len()).into_iter()
                    .map(|vertex| {
                        let p = positions_set[part][vertex];
                        Vert {
                            position: [p.x, p.y],
                            texture_uv: texture_uvs_set[part][vertex],
                        }
                    }).collect();
                let indices = triangle_indices_set[part].clone();
                let texture_index = texture_indices_set[part];
                let masks = masks_set[part].clone();
                let sc = screen_colors_set[part];
                let mc = multiply_colors_set[part];
                let mut blend = normal_blend;

                // Temporarily disabled, warning on unnecessary mutability will
                // remind me of that
                //
                // constant_flags_set[part].into_iter()
                // .for_each(|flag| match flag {
                //     Flag::BlendAdditive       => blend = add_blend,
                //     Flag::BlendMultiplicative => blend = multi_blend,
                //     _                         => {},
                // });

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
                    blend,
                }
            }).collect();

        parts.sort_by_key(|part| part.order);
        drop(dynamic);

        //                  _   _
        //  _ __ ___   ___ | |_(_) ___  _ __
        // | '_ ` _ \ / _ \| __| |/ _ \| '_ \
        // | | | | | | (_) | |_| | (_) | | | |
        // |_| |_| |_|\___/ \__|_|\___/|_| |_|

        let motions: Vec<motion::Motion> =
            motions_paths.iter()
            .map(|path| motion::Motion::new(path)).collect();

        //  _            _                       
        // | |_ _____  _| |_ _   _ _ __ ___  ___ 
        // | __/ _ \ \/ / __| | | | '__/ _ \/ __|
        // | ||  __/>  <| |_| |_| | | |  __/\__ \
        //  \__\___/_/\_\\__|\__,_|_|  \___||___/

        let textures = {
            texture_paths.into_iter()
            .map(|path| {
                let image_file = File::open(path).unwrap();
                image::load(BufReader::new(image_file),
                            image::ImageFormat::Png).unwrap()
                .to_rgba8()
            }).collect()
        };

        let mut dynamic = l2d.write_dynamic();
        dynamic.reset_drawable_dynamic_flags();
        dynamic.update();
        drop(dynamic);

        Self {
            l2d,
            parts,
            textures,
            motions,
        }
    }
}
