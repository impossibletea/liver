use std::{
    fs::File,
    iter::zip,
    collections::{HashMap, VecDeque},
};
use glium::{
    Blend,
    Display,
    Surface,
    uniform,
    DrawParameters,
    BlendingFunction,
    implement_vertex,
    program::Program,
    vertex::VertexBuffer,
    LinearBlendingFactor as F,
    index::{IndexBuffer, PrimitiveType},
    texture::{SrgbTexture2d, RawImage2d},
};
use cubism::{
    motion::Motion,
    model::UserModel,
    core::{self, ConstantFlags, DynamicFlags},
    json::{
        model::Model3,
        motion::Motion3,
    },
};
use crate::{Config, APP_NAME, CONFIG};

const BLEND_ADD:    ConstantFlags = ConstantFlags::BLEND_ADDITIVE;
const BLEND_MULT:   ConstantFlags = ConstantFlags::BLEND_MULTIPLICATIVE;
const MASK_INV:     ConstantFlags = ConstantFlags::IS_INVERTED_MASK;

const VISIBLE:          DynamicFlags = DynamicFlags::IS_VISIBLE;
const VISIBLE_CHANGED:  DynamicFlags = DynamicFlags::VISIBILITY_CHANGED;
const OPACITY_CHANGED:  DynamicFlags = DynamicFlags::OPACITY_CHANGED;
const ORDER_CHANGED:    DynamicFlags = DynamicFlags::RENDER_ORDER_CHANGED;
const VERTICES_CHANGED: DynamicFlags = DynamicFlags::VERTEX_POSITIONS_CHANGED;

//  __  __           _      _
// |  \/  | ___   __| | ___| |
// | |\/| |/ _ \ / _` |/ _ \ |
// | |  | | (_) | (_| |  __/ |
// |_|  |_|\___/ \__,_|\___|_|

pub struct Model {
    model:     UserModel,
    motions:   HashMap<String, MotionData>,
    queue:     Queue,
    canvas:    CanvasInfo,
    textures:  Vec<SrgbTexture2d>,
    drawables: Vec<Drawable>,
}

//  __  __       _   _             ____        _
// |  \/  | ___ | |_(_) ___  _ __ |  _ \  __ _| |_ __ _
// | |\/| |/ _ \| __| |/ _ \| '_ \| | | |/ _` | __/ _` |
// | |  | | (_) | |_| | (_) | | | | |_| | (_| | || (_| |
// |_|  |_|\___/ \__|_|\___/|_| |_|____/ \__,_|\__\__,_|

struct MotionData {
    motion:   Motion,
    looped:   bool,
    duration: f32,
}

//   ___
//  / _ \ _   _  ___ _   _  ___
// | | | | | | |/ _ \ | | |/ _ \
// | |_| | |_| |  __/ |_| |  __/
//  \__\_\\__,_|\___|\__,_|\___|

struct Queue {
    lineup:    VecDeque<String>,
    current:   Option<String>,
    duration:  f32,
    elapsed:   f32,
    is_paused: bool,
}

//   ____                          ___        __
//  / ___|__ _ _ ____   ____ _ ___|_ _|_ __  / _| ___
// | |   / _` | '_ \ \ / / _` / __|| || '_ \| |_ / _ \
// | |__| (_| | | | \ V / (_| \__ \| || | | |  _| (_) |
//  \____\__,_|_| |_|\_/ \__,_|___/___|_| |_|_|  \___/

struct CanvasInfo {
    size:   [f32; 2],
    origin: [f32; 2],
    scale:  f32,
}

//  ____                          _     _
// |  _ \ _ __ __ ___      ____ _| |__ | | ___
// | | | | '__/ _` \ \ /\ / / _` | '_ \| |/ _ \
// | |_| | | | (_| |\ V  V / (_| | |_) | |  __/
// |____/|_|  \__,_| \_/\_/ \__,_|_.__/|_|\___|

struct Drawable {
    vertex_buffer: VertexBuffer<Vert>,
    index_buffer:  IndexBuffer<u16>,
    visible:       bool,
    opacity:       f32,
    blend:         Blend,
    texture_index: i32,
    order:         i32,
}

//  __  __           _      _
// |  \/  | ___   __| | ___| |  _ _
// | |\/| |/ _ \ / _` |/ _ \ | (_|_)
// | |  | | (_) | (_| |  __/ |  _ _
// |_|  |_|\___/ \__,_|\___|_| (_|_)

impl Model {

    //  _ _   _ __   _____      __
    // (_|_) | '_ \ / _ \ \ /\ / /
    //  _ _  | | | |  __/\ V  V /
    // (_|_) |_| |_|\___| \_/\_/

    pub fn new(config:  &Config,
               display: &Display) -> Result<Self, String> {
        let mut model = Self::init(config,
                                   display)?;

        if let Some(q) = &config.model.motions {
            q.iter().for_each(|m| model.queue(m).unwrap_or(()));
        }

        if let Some(effect) = model.motions.get_mut("effect") {
            effect.motion.play();
        }

        Ok(model)
    }

    //        _       _ _
    //  _ _  (_)_ __ (_) |_
    // (_|_) | | '_ \| | __|
    //  _ _  | | | | | | |_
    // (_|_) |_|_| |_|_|\__|

    fn init(config:  &Config,
            display: &Display) -> Result<Self, String> {

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
            confy::get_configuration_file_path(APP_NAME,
                                               CONFIG)
            .map_err(|e| format!("Error getting assets path: {e}"))
            .and_then(|mut conf| {conf.pop(); Ok(conf)})?
            .join(&config.model.path)
            .join(&name);

        //                        _      _ _____
        //    _ __ ___   ___   __| | ___| |___ /
        //   | '_ ` _ \ / _ \ / _` |/ _ \ | |_ \
        //  _| | | | | | (_) | (_| |  __/ |___) |
        // (_)_| |_| |_|\___/ \__,_|\___|_|____/

        let model3 =
            File::open(path.join(format!("{name}.model3.json")))
            .map_err(|e| format!("Error opening json: {e}"))
            .and_then(|f| Model3::from_reader(f)
                          .map_err(|e| format!("Error parsing json: {e}")))?;

        //                        _      _
        //    _ __ ___   ___   __| | ___| |
        //   | '_ ` _ \ / _ \ / _` |/ _ \ |
        //  _| | | | | | (_) | (_| |  __/ |
        // (_)_| |_| |_|\___/ \__,_|\___|_|

        let model =
            UserModel::from_model3(&path,
                                   &model3)
            .map_err(|e| format!("Error creating model: {e}"))?;

        //                    _   _
        //    _ __ ___   ___ | |_(_) ___  _ __  ___
        //   | '_ ` _ \ / _ \| __| |/ _ \| '_ \/ __|
        //  _| | | | | | (_) | |_| | (_) | | | \__ \
        // (_)_| |_| |_|\___/ \__|_|\___/|_| |_|___/

        let mut motions = HashMap::new();

        let mut motion_files = model3.file_references.motions.azur_lane.iter();
        while let Some(m) = motion_files.next() {
            let name =
                m.file.file_name()
                .and_then(|f| f.to_str())
                .and_then(|s| s.split('.').next())
                .map(|s| s.to_string());

            let n = match name {
                Some(s) => s,
                None    => {eprintln!("Fucked up motion name"); continue}
            };

            if motions.contains_key(&n) {
                eprintln!("Duplicated motion {n}");
                continue;
            }

            let motion3 =
                File::open(path.join(&m.file))
                .map_err(|e| format!("Error opening json: {e}"))
                .and_then(|f| {
                    Motion3::from_reader(f)
                    .map_err(|e| format!("Error parsing json: {e}"))
                })?;

            let looped   = motion3.meta.looped;
            let duration = motion3.meta.duration;
            let motion   = Motion::new(motion3);

            let m = MotionData {
                looped,
                duration,
                motion,
            };

            motions.insert(n, m);
        }

        //    __ _ _   _  ___ _   _  ___
        //   / _` | | | |/ _ \ | | |/ _ \
        //  | (_| | |_| |  __/ |_| |  __/
        // (_)__, |\__,_|\___|\__,_|\___|
        //      |_|

        let queue = Queue {
            lineup:    VecDeque::new(),
            current:   None,
            duration:  0.,
            elapsed:   0.,
            is_paused: false,
        };

        //    ___ __ _ _ ____   ____ _ ___
        //   / __/ _` | '_ \ \ / / _` / __|
        //  | (_| (_| | | | \ V / (_| \__ \
        // (_)___\__,_|_| |_|\_/ \__,_|___/

        let canvas = {
            let t = model.canvas_info();

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
                model3.file_references.textures.iter()
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

        let drawables: Vec<_> =
            Result::from_iter(model.drawables()
                              .map(|d| Drawable::new(d, display)))?;

        //           _
        //  _ __ ___| |_ _   _ _ __ _ __
        // | '__/ _ \ __| | | | '__| '_ \
        // | | |  __/ |_| |_| | |  | | | |
        // |_|  \___|\__|\__,_|_|  |_| |_|

        Ok(Self {
            model,
            motions,
            queue,
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
                            program: &Program,
                            aspect:  [f32; 2]) -> Result<(), String> {
        let drawables = self.ordered();

        Result::from(
            drawables.iter()
            .filter(|d| d.visible)
            .map(|d| {
                let uniforms = uniform!{
                    size:    self.canvas.size,
                    origin:  self.canvas.origin,
                    scale:   self.canvas.scale,
                    opacity: d.opacity,
                    tex:     &self.textures[d.texture_index as usize],
                    aspect:  aspect,
                };

                let params = &DrawParameters {
                    blend: d.blend,
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

    //                        _       _
    //  _ _   _   _ _ __   __| | __ _| |_ ___
    // (_|_) | | | | '_ \ / _` |/ _` | __/ _ \
    //  _ _  | |_| | |_) | (_| | (_| | ||  __/
    // (_|_)  \__,_| .__/ \__,_|\__,_|\__\___|
    //             |_|

    pub fn update(&mut self,
                  dt: f64) -> Result<(), String> {
        let queue = &mut self.queue;
        if queue.is_paused {return Ok(())}
        queue.elapsed += dt as f32;

        if queue.elapsed >= queue.duration {
            self.next()
            .or_else(|| self.set_motion("idle"))
            .unwrap_or(());
        }

        let current = match &self.queue.current {
            Some(c) => c,
            None    => return Err(format!("No motion set"))
        };

        let motion_data =
            &mut self.motions
            .get_mut(current)
            .ok_or(format!("No motion {current}"))?;

        let motion = &mut motion_data.motion;
        motion.tick(dt);
        motion
        .update(self.model.model_mut())
        .map_err(|e| format!("Failed to update model: {e}"))?;

        if let Some(effect_data) = &mut self.motions.get_mut("effect") {
            let effect = &mut effect_data.motion;
            effect.tick(dt);
            effect
            .update(self.model.model_mut())
            .map_err(|e| format!("Failed to update model: {e}"))?;
        }

        self.model.model_mut().update();

        zip(&mut self.drawables,
            self.model.drawables())
        .for_each(|(d, s)| d.update(s));

        Ok(())
    }

    //              _
    //  _ _   _ __ | | __ _ _   _
    // (_|_) | '_ \| |/ _` | | | |
    //  _ _  | |_) | | (_| | |_| |
    // (_|_) | .__/|_|\__,_|\__, |
    //       |_|            |___/

    pub fn play(&mut self) -> Option<()> {
        self.queue.is_paused = false;
        let current = match &self.queue.current {
            Some(c) => c,
            None    => return None
        };

        self.motions
        .get_mut(current)
        .map(|m| m.motion.play())
    }

    //  _ _   _ __   __ _ _   _ ___  ___
    // (_|_) | '_ \ / _` | | | / __|/ _ \
    //  _ _  | |_) | (_| | |_| \__ \  __/
    // (_|_) | .__/ \__,_|\__,_|___/\___|
    //       |_|

    pub fn pause(&mut self) -> Option<()> {
        self.queue.is_paused = true;
        let current = match &self.queue.current {
            Some(c) => c,
            None    => return None
        };

        self.motions
        .get_mut(current)
        .map(|m| m.motion.pause())
    }

    //            _
    //  _ _   ___| |_ ___  _ __
    // (_|_) / __| __/ _ \| '_ \
    //  _ _  \__ \ || (_) | |_) |
    // (_|_) |___/\__\___/| .__/
    //                    |_|

    pub fn stop(&mut self) -> Option<()> {
        self.queue.is_paused = true;
        self.queue.elapsed = 0.;
        let current = match &self.queue.current {
            Some(c) => c,
            None    => return None
        };

        self.motions
        .get_mut(current)
        .map(|m| m.motion.stop())
    }

    //                      _             _
    //  _ _   _ __ ___  ___| |_ __ _ _ __| |_
    // (_|_) | '__/ _ \/ __| __/ _` | '__| __|
    //  _ _  | | |  __/\__ \ || (_| | |  | |_
    // (_|_) |_|  \___||___/\__\__,_|_|   \__|

    pub fn restart(&mut self) -> Option<()> {self.stop().and(self.play())}

    //                 _                     _   _
    //  _ _   ___  ___| |_   _ __ ___   ___ | |_(_) ___  _ __
    // (_|_) / __|/ _ \ __| | '_ ` _ \ / _ \| __| |/ _ \| '_ \
    //  _ _  \__ \  __/ |_  | | | | | | (_) | |_| | (_) | | | |
    // (_|_) |___/\___|\__| |_| |_| |_|\___/ \__|_|\___/|_| |_|

    fn set_motion(&mut self,
                  new: &str) -> Option<()> {
        self.stop().unwrap_or(());

        let motion_data = &mut self.motions.get_mut(new)?;

        motion_data.motion.set_looped(motion_data.looped);

        self.queue.current = Some(new.to_string());
        self.queue.duration = motion_data.duration;
        self.queue.elapsed = 0.;
        self.play();
        eprintln!("Set motion {new}");
        Some(())
    }

    //  _ _    __ _ _   _  ___ _   _  ___
    // (_|_)  / _` | | | |/ _ \ | | |/ _ \
    //  _ _  | (_| | |_| |  __/ |_| |  __/
    // (_|_)  \__, |\__,_|\___|\__,_|\___|
    //           |_|

    pub fn queue(&mut self,
                 new: &str) -> Option<()> {
        if !self.motions.contains_key(new) {return None;}

        if self.queue.current.is_some() {
            self.queue.lineup.push_back(new.to_string());
        } else {
            self.set_motion(new);
        }
        eprintln!("Queued motion {new}");
        Some(())
    }

    //                        _
    //  _ _   _ __   _____  _| |_
    // (_|_) | '_ \ / _ \ \/ / __|
    //  _ _  | | | |  __/>  <| |_
    // (_|_) |_| |_|\___/_/\_\\__|

    fn next(&mut self) -> Option<()> {
        self.queue.lineup
        .pop_front()
        .and_then(|c| self.set_motion(c.as_str()))
    }

    //                       _           _
    //  _ _   ___  ___  _ __| |_ ___  __| |
    // (_|_) / __|/ _ \| '__| __/ _ \/ _` |
    //  _ _  \__ \ (_) | |  | ||  __/ (_| |
    // (_|_) |___/\___/|_|   \__\___|\__,_|

    fn ordered(&self) -> Vec<&Drawable> {
        let mut result = Vec::from_iter(self.drawables.iter());
        result.sort_unstable_by_key(|d| d.order);
        result
    }
}

//  ____                          _     _
// |  _ \ _ __ __ ___      ____ _| |__ | | ___   _ _
// | | | | '__/ _` \ \ /\ / / _` | '_ \| |/ _ \ (_|_)
// | |_| | | | (_| |\ V  V / (_| | |_) | |  __/  _ _
// |____/|_|  \__,_| \_/\_/ \__,_|_.__/|_|\___| (_|_)

impl Drawable {

    //  _ _   _ __   _____      __
    // (_|_) | '_ \ / _ \ \ /\ / /
    //  _ _  | | | |  __/\ V  V /
    // (_|_) |_| |_|\___| \_/\_/

    fn new(drawable: core::Drawable,
           display:  &Display) -> Result<Self, String> {
        let constant_flags = drawable.constant_flags;
        let dynamic_flags = drawable.dynamic_flags;

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
            VertexBuffer::dynamic(display,
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

        //        _     _ _     _
        // __   _(_)___(_) |__ | | ___
        // \ \ / / / __| | '_ \| |/ _ \
        //  \ V /| \__ \ | |_) | |  __/
        // (_)_/ |_|___/_|_.__/|_|\___|

        let visible = dynamic_flags.contains(VISIBLE);

        //                          _ _
        //    ___  _ __   __ _  ___(_) |_ _   _
        //   / _ \| '_ \ / _` |/ __| | __| | | |
        //  | (_) | |_) | (_| | (__| | |_| |_| |
        // (_)___/| .__/ \__,_|\___|_|\__|\__, |
        //        |_|                     |___/

        let opacity = drawable.opacity;

        //    _     _                _
        //   | |__ | | ___ _ __   __| |
        //   | '_ \| |/ _ \ '_ \ / _` |
        //  _| |_) | |  __/ | | | (_| |
        // (_)_.__/|_|\___|_| |_|\__,_|

        let blend = if constant_flags.contains(BLEND_ADD) {
            Blend {
                color: BlendingFunction::Addition {
                    source:      F::SourceAlpha,
                    destination: F::One,
                },
                alpha: BlendingFunction::Addition {
                    source:      F::One,
                    destination: F::One,
                },
                constant_value: (0., 0., 0., 1.),
            }
        } else if constant_flags.contains(BLEND_MULT) {
            // TODO! implement proper multiplicative blending
            Blend {
                color: BlendingFunction::Addition {
                    source:      F::Zero,
                    destination: F::One,
                },
                alpha: BlendingFunction::Addition {
                    source:      F::Zero,
                    destination: F::One,
                },
                constant_value: (1., 1., 1., 1.),
            }
        } else {
            Blend {
                color: BlendingFunction::Addition {
                    source:      F::SourceAlpha,
                    destination: F::OneMinusSourceAlpha,
                },
                alpha: BlendingFunction::Addition {
                    source:      F::One,
                    destination: F::OneMinusSourceAlpha,
                },
                .. Default::default()
            }
        };

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

        let order = drawable.render_order;

        //           _
        //  _ __ ___| |_ _   _ _ __ _ __
        // | '__/ _ \ __| | | | '__| '_ \
        // | | |  __/ |_| |_| | |  | | | |
        // |_|  \___|\__|\__,_|_|  |_| |_|

        Ok(Drawable {
            vertex_buffer,
            index_buffer,
            visible,
            opacity,
            blend,
            texture_index,
            order,
        })
    }

    //                        _       _
    //  _ _   _   _ _ __   __| | __ _| |_ ___
    // (_|_) | | | | '_ \ / _` |/ _` | __/ _ \
    //  _ _  | |_| | |_) | (_| | (_| | ||  __/
    // (_|_)  \__,_| .__/ \__,_|\__,_|\__\___|
    //             |_|

    fn update(&mut self,
              drawable: core::Drawable) {
        let flags = drawable.dynamic_flags;

        if flags.contains(VERTICES_CHANGED) {
            let vertices: Vec<_> =
                zip(drawable.vertex_positions,
                    drawable.vertex_uvs)
                .map(|(pos, uv)| Vert{
                    position:   *pos,
                    texture_uv: *uv,
                }).collect();
            self.vertex_buffer.write(&vertices);
        }

        if flags.contains(OPACITY_CHANGED) {
            self.opacity = drawable.opacity;
        }

        if flags.contains(ORDER_CHANGED) {
            self.order = drawable.render_order;
        }

        if flags.contains(VISIBLE_CHANGED) {
            self.visible = flags.contains(VISIBLE);
        }
    }
}

// __     __        _
// \ \   / /__ _ __| |_ _____  __
//  \ \ / / _ \ '__| __/ _ \ \/ /
//   \ V /  __/ |  | ||  __/>  <
//    \_/ \___|_|   \__\___/_/\_\

#[derive(Copy, Clone, Debug)]
struct Vert {
    position:   [f32; 2],
    texture_uv: [f32; 2],
}

implement_vertex!(Vert, position, texture_uv);

