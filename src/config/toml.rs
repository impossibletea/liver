use std::error::Error;

use super::{
    Config,
    constant::*,
};

pub fn config_file() -> Result<Config, Box<dyn Error>>
{
    let config = confy::load(APP_NAME, CONFIG)?;
    Ok(config)
}

