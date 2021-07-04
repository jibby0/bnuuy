#![feature(proc_macro_hygiene, decl_macro)]
#![feature(iter_zip)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate enum_dispatch;

pub mod api;
pub mod bnuuy_config;
pub mod db;
pub mod image_sources;
pub mod logger;
pub mod schema;

use std::{error::Error, result::Result as StdResult};
pub type Result<T> = StdResult<T, Box<dyn Error>>;
