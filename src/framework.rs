use std::{
    fs::File,
    iter::zip,
    error::Error,
    collections::{HashMap, VecDeque},
};
use glium::{
    Blend,
    Surface,
    uniform,
    backend::Facade,
    BlendingFunction,
    implement_vertex,
    program::Program,
    vertex::VertexBuffer,
    LinearBlendingFactor as F,
    index::{IndexBuffer, PrimitiveType},
    texture::{SrgbTexture2d, RawImage2d},
    draw_parameters::{DrawParameters, Stencil, StencilTest, StencilOperation},
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
use crate::config::Config;

const BLEND_ADD:  ConstantFlags = ConstantFlags::BLEND_ADDITIVE;
const BLEND_MULT: ConstantFlags = ConstantFlags::BLEND_MULTIPLICATIVE;
const MASK_INV:   ConstantFlags = ConstantFlags::IS_INVERTED_MASK;

const VIS:           DynamicFlags = DynamicFlags::IS_VISIBLE;
const VIS_CHANGED:   DynamicFlags = DynamicFlags::VISIBILITY_CHANGED;
const ORDER_CHANGED: DynamicFlags = DynamicFlags::RENDER_ORDER_CHANGED;
const VERTS_CHANGED: DynamicFlags = DynamicFlags::VERTEX_POSITIONS_CHANGED;

//  __  __           _      _
// |  \/  | ___   __| | ___| |
// | |\/| |/ _ \ / _` |/ _ \ |
// | |  | | (_) | (_| |  __/ |
// |_|  |_|\___/ \__,_|\___|_|

pub struct Model {
    model:     UserModel,
    motions:   Motions,
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

type Motions = HashMap<String, MotionClass>;
type MotionClass = HashMap<String, MotionData>;

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
    lineup:    VecDeque<(String, String)>,
    current:   Option<(String, String)>,
    duration:  f32,
    elapsed:   f32,
    is_paused: bool,
    idle:      (String, String),
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
    index:         usize,
    vertex_buffer: VertexBuffer<Vert>,
    index_buffer:  IndexBuffer<u16>,
    visible:       bool,
    blend:         Blend,
    order:         i32,
    mask_inverted: bool,
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

    pub fn new<T>(config:  &Config,
                  display: &T) -> Result<Self, Box<dyn Error>>
    where T: Facade + ?Sized
    {
        let mut model = Self::init(config,
                                   display)?;

        config.model.motions.open.iter()
        .for_each(|(c, m)| model.queue((c, m)).unwrap_or(()));

        if let Some(effect) =
            model.motions
            .get_mut("")
            .and_then(|c| c.get_mut("effect")) {
            effect.motion.play();
        }

        Ok(model)
    }

    //        _       _ _
    //  _ _  (_)_ __ (_) |_
    // (_|_) | | '_ \| | __|
    //  _ _  | | | | | | |_
    // (_|_) |_|_| |_|_|\__|

    fn init<T>(config:  &Config,
               display: &T) -> Result<Self, Box<dyn Error>>
    where T: Facade + ?Sized
    {

        //    _ __   __ _ _ __ ___   ___
        //   | '_ \ / _` | '_ ` _ \ / _ \
        //  _| | | | (_| | | | | | |  __/
        // (_)_| |_|\__,_|_| |_| |_|\___|

        let name =
            config.model.name.clone()
            .ok_or("No model provided")?;

        //                _   _
        //    _ __   __ _| |_| |__
        //   | '_ \ / _` | __| '_ \
        //  _| |_) | (_| | |_| | | |
        // (_) .__/ \__,_|\__|_| |_|
        //   |_|

        let path = expanduser::expanduser(&config.model.path)?;

        //                        _      _ _____
        //    _ __ ___   ___   __| | ___| |___ /
        //   | '_ ` _ \ / _ \ / _` |/ _ \ | |_ \
        //  _| | | | | | (_) | (_| |  __/ |___) |
        // (_)_| |_| |_|\___/ \__,_|\___|_|____/

        let model3 = {
            let path = path.join(name);
            let file = File::open(path)?;
            Model3::from_reader(file)?
        };

        //                        _      _
        //    _ __ ___   ___   __| | ___| |
        //   | '_ ` _ \ / _ \ / _` |/ _ \ |
        //  _| | | | | | (_) | (_| |  __/ |
        // (_)_| |_| |_|\___/ \__,_|\___|_|

        let model = UserModel::from_model3(&path,
                                           &model3)?;

        //                    _   _
        //    _ __ ___   ___ | |_(_) ___  _ __  ___
        //   | '_ ` _ \ / _ \| __| |/ _ \| '_ \/ __|
        //  _| | | | | | (_) | |_| | (_) | | | \__ \
        // (_)_| |_| |_|\___/ \__|_|\___/|_| |_|___/

        let mut motions = HashMap::new();

        let mut motion_classes = model3.file_references.motions.iter();
        while let Some((class_name, c)) = motion_classes.next() {
            let mut class = HashMap::new();
            eprintln!("Adding motions from class \"{}\":", class_name);

            let mut motion_files = c.iter();
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

                let motion3 = {
                    let file = File::open(path.join(&m.file))?;
                    Motion3::from_reader(file)?
                };

                let looped   = motion3.meta.looped;
                let duration = motion3.meta.duration;
                let motion   = Motion::new(motion3);

                let m = MotionData {
                    looped,
                    duration,
                    motion,
                };

                eprintln!("    Added motion {n}");
                class.insert(n, m);
            }
            motions.insert(class_name.to_string(), class);
        }

        //    __ _ _   _  ___ _   _  ___
        //   / _` | | | |/ _ \ | | |/ _ \
        //  | (_| | |_| |  __/ |_| |  __/
        // (_)__, |\__,_|\___|\__,_|\___|
        //      |_|

        let idle = match config.model.motions.idle.clone() {
            Some(m) => m,
            None    => ("".to_string(),
                        "idle".to_string())
        };

        let queue = Queue {
            lineup:    VecDeque::new(),
            current:   None,
            duration:  0.,
            elapsed:   0.,
            is_paused: false,
            idle,
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

        let mut textures = Vec::new();

        let mut iter = model3.file_references.textures.iter();
        while let Some(r) = iter.next() {
            let t_path = path.join(&r);
            let image =
                image::open(&t_path)?
                .to_rgba8();
            let image_dimensions = image.dimensions();
            let image_raw =
                RawImage2d::from_raw_rgba_reversed(&image.into_raw(),
                                                   image_dimensions);
            let texture = SrgbTexture2d::new(display,
                                             image_raw)?;
            textures.push(texture);
        };

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

    pub fn draw<T>(&self,
                   frame:   &mut T,
                   program: &Program,
                   aspect:  [f32; 2]) -> Result<(), Box<dyn Error>>
    where T: Surface
    {
        let drawables = self.ordered();
        let mut iter =
            drawables
            .iter()
            .filter(|d| d.visible);

        while let Some(d) = iter.next() {
            let md = self.model.drawable_at(d.index);

            //                      _
            //  _ __ ___   __ _ ___| | __
            // | '_ ` _ \ / _` / __| |/ /
            // | | | | | | (_| \__ \   <
            // |_| |_| |_|\__,_|___/_|\_\

            frame.clear_stencil(0);
            let masks = md.masks;
            if !masks.is_empty() {
                let mut masks = masks.iter();

                while let Some(m) = masks.next() {
                    let find =
                        drawables.iter()
                        .find(|d| d.index as i32 == *m);

                    let d = match find {
                        Some(s) => s,
                        None    => {
                            eprintln!("Failed to look up mask {m}");
                            continue
                        }
                    };

                    let uniforms = uniform!{
                        size:    self.canvas.size,
                        origin:  self.canvas.origin,
                        scale:   self.canvas.scale,
                        opacity: 1. as f32,
                        tex:     &self.textures[md.texture_index as usize],
                        aspect:  aspect,
                    };

                    let op = StencilOperation::Replace;
                    let params = DrawParameters {
                        color_mask: (false, false, false, false),
                        stencil:    Stencil {
                            fail_operation_clockwise:                    op,
                            pass_depth_fail_operation_clockwise:         op,
                            depth_pass_operation_clockwise:              op,
                            fail_operation_counter_clockwise:            op,
                            pass_depth_fail_operation_counter_clockwise: op,
                            depth_pass_operation_counter_clockwise:      op,
                            reference_value_clockwise:                   1,
                            reference_value_counter_clockwise:           1,
                            .. Default::default()
                        },
                        .. Default::default()
                    };

                    frame.draw(&d.vertex_buffer,
                               &d.index_buffer,
                               program,
                               &uniforms,
                               &params)?;
                }
            }

            //      _
            //   __| |_ __ __ ___      __
            //  / _` | '__/ _` \ \ /\ / /
            // | (_| | | | (_| |\ V  V /
            //  \__,_|_|  \__,_| \_/\_/

            let uniforms = uniform!{
                size:    self.canvas.size,
                origin:  self.canvas.origin,
                scale:   self.canvas.scale,
                opacity: md.opacity,
                tex:     &self.textures[md.texture_index as usize],
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
                       &params)?;
        }

        Ok(())
    }

    //                        _       _
    //  _ _   _   _ _ __   __| | __ _| |_ ___
    // (_|_) | | | | '_ \ / _` |/ _` | __/ _ \
    //  _ _  | |_| | |_) | (_| | (_| | ||  __/
    // (_|_)  \__,_| .__/ \__,_|\__,_|\__\___|
    //             |_|

    pub fn update(&mut self,
                  dt: f64) -> Result<(), Box<dyn Error>>
    {
        let queue = &mut self.queue;
        if queue.is_paused {return Ok(())}
        queue.elapsed += dt as f32;

        if queue.elapsed >= queue.duration {self.next();}

        let current =
            &self.queue.current.clone()
            .ok_or("No motion set")?;

        let motion_data =
            &mut self.motions
            .get_mut(&current.0)
            .and_then(|class| class.get_mut(&current.1))
            .ok_or(format!("No motion {} in {}", current.1, current.0))?;

        let motion = &mut motion_data.motion;
        motion.tick(dt);
        motion
        .update(self.model.model_mut())?;

        let effect =
            &mut self.motions
            .get_mut("")
            .and_then(|c| c.get_mut("effect"));
        if let Some(effect_data) = effect {
                let effect = &mut effect_data.motion;
                effect.tick(dt);
                effect
                .update(self.model.model_mut())?;
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

    pub fn play(&mut self) -> Option<()>
    {
        self.queue.is_paused = false;
        let current = match &self.queue.current {
            Some(c) => c,
            None    => return None
        };

        self.motions
        .get_mut(&current.0)
        .and_then(|c| c.get_mut(&current.1))
        .map(|m| m.motion.play())
    }

    //  _ _   _ __   __ _ _   _ ___  ___
    // (_|_) | '_ \ / _` | | | / __|/ _ \
    //  _ _  | |_) | (_| | |_| \__ \  __/
    // (_|_) | .__/ \__,_|\__,_|___/\___|
    //       |_|

    pub fn pause(&mut self) -> Option<()>
    {
        self.queue.is_paused = true;
        let current = match &self.queue.current {
            Some(c) => c,
            None    => return None
        };

        self.motions
        .get_mut(&current.0)
        .and_then(|c| c.get_mut(&current.1))
        .map(|m| m.motion.pause())
    }

    //        _                    _
    //  _ _  | |_ ___   __ _  __ _| | ___
    // (_|_) | __/ _ \ / _` |/ _` | |/ _ \
    //  _ _  | || (_) | (_| | (_| | |  __/
    // (_|_)  \__\___/ \__, |\__, |_|\___|
    //                 |___/ |___/

    pub fn toggle(&mut self) -> Option<()>
    {
        if self.queue.is_paused {self.play()} else {self.pause()}
    }

    //            _
    //  _ _   ___| |_ ___  _ __
    // (_|_) / __| __/ _ \| '_ \
    //  _ _  \__ \ || (_) | |_) |
    // (_|_) |___/\__\___/| .__/
    //                    |_|

    pub fn stop(&mut self) -> Option<()>
    {
        self.queue.is_paused = true;
        self.queue.elapsed = 0.;
        let current = match &self.queue.current {
            Some(c) => c,
            None    => return None
        };

        self.motions
        .get_mut(&current.0)
        .and_then(|c| c.get_mut(&current.1))
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
                  new: (&str, &str)) -> Option<()>
    {
        let motion_data =
            &mut self.motions
            .get_mut(new.0)
            .and_then(|c| c.get_mut(new.1))?;

        motion_data.motion.set_looped(motion_data.looped);

        self.queue.current = Some((new.0.to_string(),
                                   new.1.to_string()));
        self.queue.duration = motion_data.duration;
        self.queue.elapsed = 0.;
        self.restart();
        eprintln!("Set motion {} from {}", new.1, new.0);
        Some(())
    }

    //                 _
    //  _ _   ___  ___| |_
    // (_|_) / __|/ _ \ __|
    //  _ _  \__ \  __/ |_
    // (_|_) |___/\___|\__|

    pub fn set(&mut self,
               new: (&str, &str)) -> Option<()>
    {
        let has_motion =
            self.motions.get(new.0)
            .map(|c| c.contains_key(new.1))
            .unwrap_or(false);

        if !has_motion {return None}

        self.set_motion(new)
    }

    //  _ _    __ _ _   _  ___ _   _  ___
    // (_|_)  / _` | | | |/ _ \ | | |/ _ \
    //  _ _  | (_| | |_| |  __/ |_| |  __/
    // (_|_)  \__, |\__,_|\___|\__,_|\___|
    //           |_|

    pub fn queue(&mut self,
                 new: (&str, &str)) -> Option<()>
    {
        let has_motion =
            self.motions.get(new.0)
            .map(|c| c.contains_key(new.1))
            .unwrap_or(false);

        if !has_motion {return None}

        if self.queue.current.is_some() {
            self.queue.lineup.push_back((new.0.to_string(),
                                         new.1.to_string()));
            eprintln!("Queued motion {} from {}", new.1, new.0);
        } else {
            self.set_motion(new)?;
        }
        Some(())
    }

    //                        _
    //  _ _   _ __   _____  _| |_
    // (_|_) | '_ \ / _ \ \/ / __|
    //  _ _  | | | |  __/>  <| |_
    // (_|_) |_| |_|\___/_/\_\\__|

    fn next(&mut self) -> Option<()>
    {
        let next = match self.queue.lineup.pop_front() {
            Some(m) => m,
            None    => self.queue.idle.clone()
        };

        self.set_motion((next.0.as_str(),
                         next.1.as_str()))
    }

    //                      _                   _
    //  _ _    ___  _ __ __| | ___ _ __ ___  __| |
    // (_|_)  / _ \| '__/ _` |/ _ \ '__/ _ \/ _` |
    //  _ _  | (_) | | | (_| |  __/ | |  __/ (_| |
    // (_|_)  \___/|_|  \__,_|\___|_|  \___|\__,_|

    fn ordered(&self) -> Vec<&Drawable>
    {
        let mut result = Vec::from_iter(self.drawables.iter());
        result.sort_unstable_by_key(|d| d.order);
        result
    }

    //            _
    //  _ _   ___(_)_______
    // (_|_) / __| |_  / _ \
    //  _ _  \__ \ |/ /  __/
    // (_|_) |___/_/___\___|

    pub fn size(&self) -> [f32; 2] {self.canvas.size}
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

    fn new<T>(drawable: core::Drawable,
              display:  &T) -> Result<Self, Box<dyn Error>>
    where T: Facade + ?Sized
    {
        let constant_flags = drawable.constant_flags;
        let dynamic_flags = drawable.dynamic_flags;

        //    _           _
        //   (_)_ __   __| | _____  __
        //   | | '_ \ / _` |/ _ \ \/ /
        //  _| | | | | (_| |  __/>  <
        // (_)_|_| |_|\__,_|\___/_/\_\

        let index = drawable.index;

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
                                  &vertices)?;

        //    _           _             _            __  __
        //   (_)_ __   __| | _____  __ | |__  _   _ / _|/ _| ___ _ __
        //   | | '_ \ / _` |/ _ \ \/ / | '_ \| | | | |_| |_ / _ \ '__|
        //  _| | | | | (_| |  __/>  <  | |_) | |_| |  _|  _|  __/ |
        // (_)_|_| |_|\__,_|\___/_/\_\ |_.__/ \__,_|_| |_|  \___|_|

        let index_buffer =
            IndexBuffer::new(display,
                             PrimitiveType::TrianglesList,
                             drawable.indices)?;

        //        _     _ _     _
        // __   _(_)___(_) |__ | | ___
        // \ \ / / / __| | '_ \| |/ _ \
        //  \ V /| \__ \ | |_) | |  __/
        // (_)_/ |_|___/_|_.__/|_|\___|

        let visible = dynamic_flags.contains(VIS);

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

        //                       _                           _
        //    _ __ ___ _ __   __| | ___ _ __    ___  _ __ __| | ___ _ __
        //   | '__/ _ \ '_ \ / _` |/ _ \ '__|  / _ \| '__/ _` |/ _ \ '__|
        //  _| | |  __/ | | | (_| |  __/ |    | (_) | | | (_| |  __/ |
        // (_)_|  \___|_| |_|\__,_|\___|_|     \___/|_|  \__,_|\___|_|

        let order = drawable.render_order;

        //                        _      _                     _           _
        //    _ __ ___   __ _ ___| | __ (_)_ ____   _____ _ __| |_ ___  __| |
        //   | '_ ` _ \ / _` / __| |/ / | | '_ \ \ / / _ \ '__| __/ _ \/ _` |
        //  _| | | | | | (_| \__ \   <  | | | | \ V /  __/ |  | ||  __/ (_| |
        // (_)_| |_| |_|\__,_|___/_|\_\ |_|_| |_|\_/ \___|_|   \__\___|\__,_|

        let mask_inverted = constant_flags.contains(MASK_INV);

        //           _
        //  _ __ ___| |_ _   _ _ __ _ __
        // | '__/ _ \ __| | | | '__| '_ \
        // | | |  __/ |_| |_| | |  | | | |
        // |_|  \___|\__|\__,_|_|  |_| |_|

        Ok(Drawable {
            index,
            vertex_buffer,
            index_buffer,
            visible,
            blend,
            order,
            mask_inverted,
        })
    }

    //                        _       _
    //  _ _   _   _ _ __   __| | __ _| |_ ___
    // (_|_) | | | | '_ \ / _` |/ _` | __/ _ \
    //  _ _  | |_| | |_) | (_| | (_| | ||  __/
    // (_|_)  \__,_| .__/ \__,_|\__,_|\__\___|
    //             |_|

    fn update(&mut self,
              drawable: core::Drawable)
    {
        let flags = drawable.dynamic_flags;

        if flags.contains(VERTS_CHANGED) {
            let vertices: Vec<_> =
                zip(drawable.vertex_positions,
                    drawable.vertex_uvs)
                .map(|(pos, uv)| Vert{
                    position:   *pos,
                    texture_uv: *uv,
                }).collect();
            self.vertex_buffer.write(&vertices);
        }

        if flags.contains(ORDER_CHANGED) {self.order = drawable.render_order;}
        if flags.contains(VIS_CHANGED)   {self.visible = flags.contains(VIS);}
    }
}

// __     __        _
// \ \   / /__ _ __| |_ _____  __
//  \ \ / / _ \ '__| __/ _ \ \/ /
//   \ V /  __/ |  | ||  __/>  <
//    \_/ \___|_|   \__\___/_/\_\

#[derive(Copy, Clone, Debug)]
pub struct Vert {
    position:   [f32; 2],
    texture_uv: [f32; 2],
}

implement_vertex!(Vert, position, texture_uv);

impl Vert {
    pub fn new(position:   [f32; 2],
               texture_uv: [f32; 2]) -> Self
    {
        Self {
            position,
            texture_uv,
        }
    }
}
