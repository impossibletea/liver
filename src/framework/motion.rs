use std::path::Path;

mod motion_json;

struct MotionData {
    duration: f32,
    r#loop: bool,
    fps: f32,
    curves: Vec<MotionCurve>,
    segments: Vec<MotionSegment>,
    points: Vec<MotionPoint>,
    events: Vec<MotionEvent>,
}

#[derive(Clone)]
struct MotionCurve {
    r#type: MotionCurveTarget,
    id: String,
    base_segment_index: usize,
    segment_count: usize,
    fade_in_time: f32,
    fade_out_time: f32,
}

#[derive(Copy, Clone)]
enum MotionCurveTarget {
    Model,
    Parameter,
    PartOpacity,
}

#[derive(Copy, Clone)]
struct MotionSegment {
    base_point_index: usize,
    segment_type: SegmentType,
}

#[derive(Copy, Clone)]
enum SegmentType {
    Linear,
    Bezier,
    Stepped,
    InverseStepped,
}

#[derive(Copy, Clone)]
struct MotionPoint {
    time: f32,
    value: f32,
}

struct MotionEvent {
    fire_time: f32,
    value: String,
}

impl MotionData {

    //                  _             _
    //   _____   ____ _| |_   _  __ _| |_ ___
    //  / _ \ \ / / _` | | | | |/ _` | __/ _ \
    // |  __/\ V / (_| | | |_| | (_| | ||  __/
    //  \___| \_/ \__,_|_|\__,_|\__,_|\__\___|

    fn evaluate_curve(&self,
                      index: usize,
                      time: f32) -> f32 {
        let curve = &self.curves[index];

        let mut target: Option<usize> = None;
        let total_segment_count =
            curve.base_segment_index + curve.segment_count;
        let mut point_position = 0;

        for i in curve.base_segment_index..total_segment_count {
            point_position = {
                let segment = self.segments[i];
                let a = segment.base_point_index;
                let b = match segment.segment_type {
                    SegmentType::Bezier => 3,
                    _                   => 1,
                };
                a + b
            };
            if self.points[point_position].time > time {
                target = Some(i);
                break
            }
        }

        match target {
            Some(index) => {
                let segment = self.segments[index];
                segment.evaluate(&self.points, time)
            }
            None => self.points[point_position].value
        }
    }
}

impl MotionSegment {
    pub fn evaluate(&self, p: &Vec<MotionPoint>, t: f32) -> f32 {
        type T = SegmentType;

        let b = self.base_point_index;

        match self.segment_type {
            T::Linear => {
                let ps = &[p[b], p[b + 1]];
                linear(ps, t)
            }
            T::Bezier => {
                let ps = &[p[b], p[b + 1], p[b + 2], p[b + 3]];
                bezier(ps, t)
            }
            T::Stepped => {
                let ps = &[p[b], p[b + 1]];
                stepped(ps)
            }
            T::InverseStepped => {
                let ps = &[p[b], p[b + 1]];
                inverse_stepped(ps)
            }
        }
    }

}

//  _       _                        _
// (_)_ __ | |_ ___ _ __ _ __   ___ | |
// | | '_ \| __/ _ \ '__| '_ \ / _ \| |
// | | | | | ||  __/ |  | |_) | (_) | |
// |_|_| |_|\__\___|_|  | .__/ \___/|_|
//                      |_|

fn lerp(a: &MotionPoint,
        b: &MotionPoint,
        t: f32) -> MotionPoint {
    let time  = a.time  + ((b.time  - a.time)  * t);
    let value = a.value + ((b.value - a.value) * t);

    MotionPoint {
        time,
        value,
    }
}

fn linear(points: &[MotionPoint; 2],
          time: f32) -> f32 {
    let mut t = (time - points[0].time) / (points[1].time - points[0].time);
    if t < 0. {t = 0.};

    points[0].value + ((points[1].value - points[0].value) * t)
}

fn bezier(points: &[MotionPoint; 4],
          time: f32) -> f32 {
    let mut t = (time - points[0].time) / (points[1].time - points[0].time);
    if t < 0. {t = 0.};

    let p01 = lerp(&points[0], &points[1], t);
    let p12 = lerp(&points[1], &points[2], t);
    let p23 = lerp(&points[2], &points[3], t);

    let p012 = lerp(&p01, &p12, t);
    let p123 = lerp(&p12, &p23, t);

    lerp(&p012, &p123, t).value
}

fn stepped(points: &[MotionPoint; 2]) -> f32 {points[0].value}

fn inverse_stepped(points: &[MotionPoint; 2]) -> f32 {points[1].value}

//                  _   _
//  _ __ ___   ___ | |_(_) ___  _ __
// | '_ ` _ \ / _ \| __| |/ _ \| '_ \
// | | | | | | (_) | |_| | (_) | | | |
// |_| |_| |_|\___/ \__|_|\___/|_| |_|

pub struct Motion {
    a_motion: AMotion,
    source_frame_rate: f32,
    loop_duration_seconds: f32,
    is_loop: bool,
    is_loop_fade_in: bool,
    last_weight: f32,
    motion_data: Option<MotionData>,
    eye_blink_parameter_ids: Option<Vec<usize>>,
    lip_sync_parameter_ids: Option<Vec<usize>>,
    model_curve_id_eye_blink: Option<usize>,
    model_curve_id_lip_sync: Option<usize>,
    model_curve_id_opacity: Option<usize>,
    model_opacity: f32,
}

struct AMotion {
    fade_in_seconds: f32,
    fade_out_seconds: f32,
    weight: f32,
    offset_seconds: f32,
    fired_event_values: Option<Vec<String>>,
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            source_frame_rate: 30.,
            loop_duration_seconds: -1.,
            is_loop: false,
            is_loop_fade_in: true,
            last_weight: 0.,
            motion_data: None,
            eye_blink_parameter_ids: None,
            lip_sync_parameter_ids: None,
            model_curve_id_eye_blink: None,
            model_curve_id_lip_sync: None,
            model_curve_id_opacity: None,
            model_opacity: 1.,
            a_motion: AMotion::default(),
        }
    }
}

impl Default for AMotion {
    fn default() -> Self {
        Self {
            fade_in_seconds: -1.,
            fade_out_seconds: -1.,
            weight: 1.,
            offset_seconds: 0.,
            fired_event_values: None,
        }
    }
}

impl Motion {

    //  _ __   _____      __
    // | '_ \ / _ \ \ /\ / /
    // | | | |  __/\ V  V /
    // |_| |_|\___| \_/\_/

    pub fn new(file_path: &Path) -> Self {
        use motion_json::JsonMotion;

        let json = JsonMotion::new(file_path);

        let duration    = json.Meta.Duration;
        let r#loop      = json.Meta.Loop;
        let fps         = json.Meta.Fps;

        let fade_in_seconds = match json.Meta.FadeInTime {
            Some(s) if s >= 0. => s,
            _                  => 1.,
        };
        let fade_out_seconds = match json.Meta.FadeOutTime {
            Some(s) if s >= 0. => s,
            _                  => 1.,
        };

        let mut curves:   Vec<MotionCurve>   = Vec::new();
        let mut segments: Vec<MotionSegment> = Vec::new();
        let mut points:   Vec<MotionPoint>   = Vec::new();

        let mut total_point_count = 0;
        let mut total_segment_count = 0;

        //   ___ _   _ _ ____   _____  ___
        //  / __| | | | '__\ \ / / _ \/ __|
        // | (__| |_| | |   \ V /  __/\__ \
        //  \___|\__,_|_|    \_/ \___||___/

        json.Curves.into_iter()
        .for_each(|curve| {
            type T = MotionCurveTarget;

            let r#type = match curve.Target.as_str() {
                "Model"       => T::Model,
                "Parameter"   => T::Parameter,
                "PartOpacity" => T::PartOpacity,
                // In the C++ Framework enum is used, and value is
                // 0-initialized, so what is technically unreachable
                // becomes Model
                _             => T::Model,
            };
            let id = curve.Id;
            let base_segment_index = total_segment_count;
            let fade_in_time = match curve.FadeInTime {
                Some(s) => s,
                None    => -1.,
            };
            let fade_out_time = match curve.FadeOutTime {
                Some(s) => s,
                None    => -1.,
            };

            let mut segments_iter = curve.Segments.into_iter();
            let mut segment_count = 1;

            let time = segments_iter.next().unwrap();
            let value = segments_iter.next().unwrap();

            points.push(MotionPoint {
                time,
                value,
            });

            total_point_count += 1;

            //                                      _
            //  ___  ___  __ _ _ __ ___   ___ _ __ | |_ ___
            // / __|/ _ \/ _` | '_ ` _ \ / _ \ '_ \| __/ __|
            // \__ \  __/ (_| | | | | | |  __/ | | | |_\__ \
            // |___/\___|\__, |_| |_| |_|\___|_| |_|\__|___/
            //           |___/

            while let Some(segment) = segments_iter.next() {
                type T = SegmentType;

                let base_point_index = total_point_count - 1;

                match segment {
                    s if s == 0. => {
                        let segment_type = T::Linear;
                        let time = segments_iter.next().unwrap();
                        let value = segments_iter.next().unwrap();

                        segments.push(MotionSegment {
                            base_point_index,
                            segment_type,
                        });

                        points.push(MotionPoint {
                            time,
                            value,
                        });

                        total_point_count += 1;
                    }
                    s if s == 1. => {
                        let segment_type = T::Bezier;

                        segments.push(MotionSegment {
                            base_point_index,
                            segment_type,
                        });

                        for _ in 0..3 {
                            let time = segments_iter.next().unwrap();
                            let value = segments_iter.next().unwrap();

                            points.push(MotionPoint {
                                time,
                                value,
                            });
                        }

                        total_point_count += 3;
                    }
                    s if s == 2. => {
                        let segment_type = T::Stepped;

                        let time = segments_iter.next().unwrap();
                        let value = segments_iter.next().unwrap();

                        segments.push(MotionSegment {
                            base_point_index,
                            segment_type,
                        });

                        points.push(MotionPoint {
                            time,
                            value,
                        });

                        total_point_count += 1;
                    }
                    s if s == 3. => {
                        let segment_type = T::InverseStepped;

                        let time = segments_iter.next().unwrap();
                        let value = segments_iter.next().unwrap();

                        segments.push(MotionSegment {
                            base_point_index,
                            segment_type,
                        });

                        points.push(MotionPoint {
                            time,
                            value,
                        });

                        total_point_count += 1;
                    }
                    _  => unreachable!()
                }

                segment_count += 1;
                total_segment_count += 1;
            }

            curves.push(MotionCurve {
                r#type,
                id,
                base_segment_index,
                fade_in_time,
                fade_out_time,
                segment_count,
            });
        });

        //                       _
        //   _____   _____ _ __ | |_ ___
        //  / _ \ \ / / _ \ '_ \| __/ __|
        // |  __/\ V /  __/ | | | |_\__ \
        //  \___| \_/ \___|_| |_|\__|___/

        let events: Vec<MotionEvent> =
            json.UserData.into_iter()
            .map(|event| {
                let fire_time = event.Time;
                let value = event.Value;

                MotionEvent {
                    fire_time,
                    value,
                }
            }).collect();

        let motion_data = MotionData {
            duration,
            r#loop,
            fps,
            curves,
            segments,
            points,
            events,
        };

        let source_frame_rate = motion_data.fps;
        let loop_duration_seconds = motion_data.duration;

        let a_motion = AMotion {
            fade_in_seconds,
            fade_out_seconds,
            .. Default::default()
        };

        Self {
            motion_data: Some(motion_data),
            source_frame_rate,
            loop_duration_seconds,
            a_motion,
            .. Default::default()
        }
    }
}
