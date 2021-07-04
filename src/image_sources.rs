pub mod instagram;
use crate::{db, Result};
use serde::{Deserialize, Serialize};

#[enum_dispatch]
pub trait ImageSource {
    fn update_image_paths(&self, pool: &mut db::Pool) -> Result<()>;
}

#[enum_dispatch(ImageSource)]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ImageSourceTypes {
    #[serde(rename = "instagram")]
    Instagram(instagram::Instagram),
    #[serde(rename = "dummy")]
    Dummy(Dummy),
}

/// Just for testing with > 1 source type
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dummy {}
impl ImageSource for Dummy {
    fn update_image_paths(&self, _: &mut db::Pool) -> Result<()> {
        Ok(())
    }
}