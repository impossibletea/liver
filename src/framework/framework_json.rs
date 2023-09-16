use std::{
    fs::File,
    path::Path,
    io::BufReader,
};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct JsonModel {
    pub Version: usize,
    pub FileReferences: JsonFileReferences,
    pub Groups: Vec<JsonGroup>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct JsonFileReferences {
    pub Moc: String,
    pub Textures: Vec<String>,
    pub Physics: String,
    pub Motions: serde_json::Value,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct JsonGroup {
    pub Target: String,
    pub Name: String,
    pub Ids: Vec<String>,
}

impl JsonModel {
    pub fn new(file_path: &Path) -> Result<Self, String> {
        File::open(file_path)
        .map_err(|e| format!("{:?}", e))
        .and_then(|file| Ok(BufReader::new(file)))
        .and_then(|reader| serde_json::from_reader(reader)
                           .map_err(|e| format!("{:?}", e)))
    }
}
