use std::{
    fs::File,
    iter::zip,
    path::Path,
    collections::HashMap,
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
use l2d::ParameterType;
use crate::logging::*;

mod motion;
mod framework_json;
use framework_json::JsonModel;

//  ____            _
// |  _ \ __ _ _ __| |_
// | |_) / _` | '__| __|
// |  __/ (_| | |  | |_
// |_|   \__,_|_|   \__|

#[derive(Debug)]
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

//  __  __           _      _
// |  \/  | ___   __| | ___| |
// | |\/| |/ _ \ / _` |/ _ \ |
// | |  | | (_) | (_| |  __/ |
// |_|  |_|\___/ \__,_|\___|_|

pub struct Model {
    pub l2d: l2d::Model,
    pub parts: Vec<Part>,
    pub textures: Vec<image::RgbaImage>,
    pub motions: Vec<motion::Motion>,
    pub current_motion: usize,
    pub last_time: f32,
    pub opacity: f32,
    pub parameters: HashMap<String, ModelParameter>,
}

pub struct ModelParameter {
    value: f32,
    max: f32,
    min: f32,
    r#type: ModelParameterType,
    keys: Vec<f32>,
    index: usize,
}

enum ModelParameterType {
    Normal,
    BlendShape,
}

impl Model {
    //  _ __   _____      __
    // | '_ \ / _ \ \ /\ / /
    // | | | |  __/\ V  V /
    // |_| |_|\___| \_/\_/

    pub fn new(path: &Path,
               name: &str) -> Result<Self, String> {

        //    _
        //   (_)___  ___  _ __
        //   | / __|/ _ \| '_ \
        //   | \__ \ (_) | | | |
        //  _/ |___/\___/|_| |_|
        // |__/

        let base_path = path.join(name);
        let json_path = base_path.join(name.to_string() + ".model3.json");
        let json = JsonModel::new(&json_path)?;
        info("Loaded model json");

        let moc_path = base_path.join(json.FileReferences.Moc);
        let texture_paths: Vec<_> =
            json.FileReferences.Textures.iter()
            .map(|texture_path| base_path.join(texture_path)).collect();

        let motions_paths: Vec<_> =
            json.FileReferences.Motions[""]
            .as_array()
            .unwrap_or_else(|| unreachable!()).iter()
            .map(|motion| {
                let motion_path =
                    motion["File"].as_str()
                    .unwrap_or_else(|| unreachable!());
                base_path.join(motion_path)
            }).collect();

        //  _ _           ____     _
        // | (_)_   _____|___ \ __| |
        // | | \ \ / / _ \ __) / _` |
        // | | |\ V /  __// __/ (_| |
        // |_|_| \_/ \___|_____\__,_|

        let l2d = {
            use l2d::{CubismCore, Model as L2DModel};

            let mut bytes = Vec::new();
            File::open(moc_path).map_err(|e| format!("{:?}", e))
            .and_then(|mut file| file.read_to_end(&mut bytes)
                             .map_err(|e| format!("{:?}", e)))
            .and_then(|_| CubismCore::default()
                          .moc_from_bytes(&bytes)
                          .map_err(|e| format!("{:?}", e)))
            .and_then(|moc| Ok(L2DModel::from_moc(&moc)))
        }?;
        info("Loaded l2d model");

        //                                       _
        //  _ __   __ _ _ __ __ _ _ __ ___   ___| |_ ___ _ __ ___
        // | '_ \ / _` | '__/ _` | '_ ` _ \ / _ \ __/ _ \ '__/ __|
        // | |_) | (_| | | | (_| | | | | | |  __/ ||  __/ |  \__ \
        // | .__/ \__,_|_|  \__,_|_| |_| |_|\___|\__\___|_|  |___/
        // |_|

        let mut parameters = HashMap::<String, ModelParameter>::new();
        let mut index = 0;

        l2d.parameters().iter()
        .for_each(|parameter| {
            let id = parameter.id().to_string();
            let value = parameter.default_value();
            let (max, min) = parameter.value_range();
            let r#type = match parameter.ty() {
                ParameterType::Normal =>
                    ModelParameterType::Normal,
                ParameterType::BlendShape =>
                    ModelParameterType::BlendShape
            };
            let keys: Vec<f32> = parameter.keys().to_owned();

            let p = ModelParameter {
                value,
                max,
                min,
                r#type,
                keys,
                index,
            };

            parameters.insert(id, p);
            index += 1;
        });

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

        // Temporarily disabled, warning on unnecessary mutability will
        // remind me of that

        //let add_blend = {
        //    let add_fn = BlendingFunction::Addition {
        //        source: Factor::One,
        //        destination: Factor::One,
        //    };
        //    Blend {
        //        color: add_fn,
        //        alpha: add_fn,
        //        constant_value,
        //    }
        //};

        //let multi_blend = {
        //    let multi_fn = BlendingFunction::Addition {
        //        source: Factor::DestinationColor,
        //        destination: Factor::OneMinusSourceAlpha,
        //    };
        //    let multi_alpha = BlendingFunction::Addition {
        //        source: Factor::Zero,
        //        destination: Factor::One,
        //    };
        //        Blend {
        //        color: multi_fn,
        //        alpha: multi_alpha,
        //        constant_value,
        //    }
        //};

        // DYNAMIC PARTS
        let dynamic = l2d.read_dynamic();

        let positions_set = dynamic.drawable_vertex_position_containers();
        let opacities_set = dynamic.drawable_opacities();
        let orders_set = dynamic.drawable_render_orders();
        let screen_colors_set = dynamic.drawable_screen_colors();
        let multiply_colors_set = dynamic.drawable_multiply_colors();

        let parts: Vec<Part> =
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
                let blend = normal_blend;

                //constant_flags_set[part].into_iter()
                //.for_each(|flag| match flag {
                //    Flag::BlendAdditive       => blend = add_blend,
                //    Flag::BlendMultiplicative => blend = multi_blend,
                //    _                         => {},
                //});

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

        drop(dynamic);

        //                  _   _
        //  _ __ ___   ___ | |_(_) ___  _ __
        // | '_ ` _ \ / _ \| __| |/ _ \| '_ \
        // | | | | | | (_) | |_| | (_) | | | |
        // |_| |_| |_|\___/ \__|_|\___/|_| |_|

        let mut motions = Vec::<motion::Motion>::new();
        let mut motions_iter = motions_paths.iter();
        while let Some(path) = motions_iter.next() {
            match motion::Motion::new(path) {
                Ok(m) => {
                    let message =
                        format!("Loaded motion from {}", path.display());
                    info(&message);
                    motions.push(m);
                }
                Err(e) => warn("Unable to load motion", e)
            }
        }

        //  _            _
        // | |_ _____  _| |_ _   _ _ __ ___  ___
        // | __/ _ \ \/ / __| | | | '__/ _ \/ __|
        // | ||  __/>  <| |_| |_| | | |  __/\__ \
        //  \__\___/_/\_\\__|\__,_|_|  \___||___/

        let mut textures_iter = texture_paths.into_iter();
        let mut textures: Vec<image::RgbaImage> = Vec::new();
        while let Some(path) = textures_iter.next() {
                File::open(path).map_err(|e| format!("{}", e))
                .and_then(|file| image::load(BufReader::new(file),
                                             image::ImageFormat::Png)
                                 .map_err(|e| format!("{}", e)))
                .and_then(|image| Ok(textures.push(image.to_rgba8())))?;
        };

        let mut dynamic = l2d.write_dynamic();
        dynamic.reset_drawable_dynamic_flags();
        dynamic.update();
        drop(dynamic);

        let current_motion = 8;
        let last_time = 0.;
        let opacity = 1.;

        Ok(Self {
            l2d,
            parts,
            textures,
            motions,
            last_time,
            parameters,
            opacity,
            current_motion,
        })
    }

    //                  _       _
    //  _   _ _ __   __| | __ _| |_ ___
    // | | | | '_ \ / _` |/ _` | __/ _ \
    // | |_| | |_) | (_| | (_| | ||  __/
    //  \__,_| .__/ \__,_|\__,_|\__\___|
    //       |_|

    pub fn update(&mut self, time: f32) {
        use motion::MotionCurveTarget as T;

        let motion = &self.motions[self.current_motion];

        let time_offset_seconds = {
            let mut offset = time - self.last_time;
            let duration = motion.loop_duration_seconds;
            if motion.is_loop {
                while offset > duration {
                    offset -= duration;
                }
            }
            if offset < 0. {0.} else {offset}
        };

        //   __           _
        //  / _| __ _  __| | ___
        // | |_ / _` |/ _` |/ _ \
        // |  _| (_| | (_| |  __/
        // |_|  \__,_|\__,_|\___|

        // todo

        //   ___ _   _ _ ____   _____  ___
        //  / __| | | | '__\ \ / / _ \/ __|
        // | (__| |_| | |   \ V /  __/\__ \
        //  \___|\__,_|_|    \_/ \___||___/

        let curves = match &motion.motion_data {
            Some(d) => &d.curves,
            None    => todo!(),
        };

        let mut curves_iter = curves.iter();
        while let Some(curve) = curves_iter.next() {
            let value =
                motion.motion_data.as_ref().unwrap()
                .evaluate_curve(curve,
                                time_offset_seconds);

            match curve.r#type {
                T::Model => {
                    // this should work with blinking, lipsync and opacity,
                    // but I am yet to figure out how
                }
                T::Parameter => {
                    let id = &curve.id;
                    let parameter = match self.parameters.get_mut(id) {
                        Some(p) => p,
                        None    => continue
                    };
                    parameter.value = value;
                }
                T::PartOpacity => {
                    let id = &curve.id;
                    let parameter = match self.parameters.get_mut(id) {
                        Some(p) => p,
                        None    => continue
                    };
                    parameter.value = value;
                }
            }
        }

        //                                       _
        //  _ __   __ _ _ __ __ _ _ __ ___   ___| |_ ___ _ __ ___
        // | '_ \ / _` | '__/ _` | '_ ` _ \ / _ \ __/ _ \ '__/ __|
        // | |_) | (_| | | | (_| | | | | | |  __/ ||  __/ |  \__ \
        // | .__/ \__,_|_|  \__,_|_| |_| |_|\___|\__\___|_|  |___/
        // _|

        let pars: Vec<(usize, f32)> =
            self.parameters.iter()
            .map(|parameter| {
                let index = parameter.1.index;
                let value = parameter.1.value;
                (index, value)
            }).collect();

        //                   _
        //  _ __   __ _ _ __| |_ ___
        // | '_ \ / _` | '__| __/ __|
        // | |_) | (_| | |  | |_\__ \
        // | .__/ \__,_|_|   \__|___/
        // |_|

        let mut dynamic = self.l2d.write_dynamic();
        let l2d_parameters = dynamic.parameter_values_mut();

        pars.iter()
        .for_each(|(i, v)| {
            l2d_parameters[*i] = *v;
        });
        dynamic.update();

        let positions_set = dynamic.drawable_vertex_position_containers();
        let opacities_set = dynamic.drawable_opacities();
        let orders_set = dynamic.drawable_render_orders();
        let screen_colors_set = dynamic.drawable_screen_colors();
        let multiply_colors_set = dynamic.drawable_multiply_colors();

        let new_values = (0..positions_set.len()).into_iter();
        let parts = &mut self.parts;

        let updates = zip(parts, new_values);

        for (part, update) in updates {
            let vertex_updates =
                (0..positions_set[update].len()).into_iter();
            let vertices = &mut part.vertices;
            let updates2 = zip(vertices, vertex_updates);

            for (vertex, update2) in updates2 {
                let p = positions_set[update][update2];
                vertex.position = [p.x, p.y];
            }

            let sc = screen_colors_set[update];
            let mc = multiply_colors_set[update];

            // Temporarily disabled, warning on unnecessary mutability will
            // remind me of that
            //
            // constant_flags_set[part].into_iter()
            // .for_each(|flag| match flag {
            //     Flag::BlendAdditive       => blend = add_blend,
            //     Flag::BlendMultiplicative => blend = multi_blend,
            //     _                         => {},
            // });

            part.screen_color = [sc.x, sc.y, sc.z, sc.w];
            part.multiply_color = [mc.x, mc.y, mc.z, mc.w];
            part.opacity = opacities_set[update];
            part.order = orders_set[update];
        }

    }

    //                 _
    //  ___  ___  _ __| |_
    // / __|/ _ \| '__| __|
    // \__ \ (_) | |  | |_
    // |___/\___/|_|   \__|

    pub fn parts_sorted(&self) -> Vec<&Part> {
        let mut result: Vec<&Part> = self.parts.iter().collect();
        result.sort_by_key(|part| part.order);
        result
    }
}
