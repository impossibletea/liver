use std::{
    fs::File,
    path::Path,
    io::BufReader,
};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct JsonMotion {
    pub Version: usize,
    pub Meta: JsonMetaInfo,
    pub Curves: Vec<JsonCurve>,
    pub UserData: Vec<JsonUserData>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct JsonMetaInfo {
    pub Duration: f32,
    pub Fps: f32,
    pub Loop: bool,
    pub CurveCount: usize,
    pub TotalSegmentCount: usize,
    pub TotalPointCount: usize,
    pub UserDataCount: usize,
    pub TotalUserDataSize: usize,
    pub FadeInTime: Option<f32>,
    pub FadeOutTime: Option<f32>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct JsonCurve {
    pub Target: String,
    pub Id: String,
    pub Segments: Vec<f32>,
    pub FadeInTime: Option<f32>,
    pub FadeOutTime: Option<f32>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct JsonUserData {
    pub Time: f32,
    pub Value: String,
}

impl JsonMotion {
    pub fn new(file_path: &Path) -> Result<Self, String> {
        File::open(file_path)
        .map_err(|e| format!("{:?}", e))
        .and_then(|file| Ok(BufReader::new(file)))
        .and_then(|reader| serde_json::from_reader(reader)
                           .map_err(|e| format!("{:?}", e)))
    }
}

