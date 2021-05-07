#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;

pub mod api;
pub mod db;
pub mod instagram;
pub mod logger;
pub mod schema;

use std::{error::Error, result::Result as StdResult};
type Result<T> = StdResult<T, Box<dyn Error>>;

const INSTA_USERNAME: &str = "MY_INSTA_USERNAME";
const INSTA_PASSWORD: &str = "MY_INSTA_PASSWORD";
static INSTA_PAGES: &[&str] = &[
    "hlee2433",
    "_prince_irvin_",
    "bbaeggomi._",
    "bobo.ellie.buns",
    "boubou_beliss_pomeranians",
    "angpang_smile",
    "shila_the_pom",
    "bulldogdays",
    "sneakersthecorgi",
];
const IMAGE_FOLDER: &str = "./cache/instagram-scraper";
