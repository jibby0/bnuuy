use crate::{image_sources::ImageSourceTypes, Result};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize)]
pub struct BnuuyConfig {
    // pub sources: Vec<Box<dyn ImageSource>>,
    pub sources: Vec<ImageSourceTypes>,
}

pub fn read_config() -> Result<BnuuyConfig> {
    // TODO add this as a CLI arg
    let mut f = File::open("bnuuy.toml")?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    Ok(toml::from_str(&contents)?)
}
