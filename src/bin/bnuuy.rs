extern crate bnuuy;
use bnuuy::{api, db, instagram, logger};
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate rocket;

use clokwerk::{Scheduler, TimeUnits};
use diesel::{Connection, SqliteConnection};

use std::thread;

embed_migrations!();

fn main() {
    let _ = dotenv::dotenv();
    logger::setup_logging(log::LevelFilter::Debug).expect("failed to initialize logging");

    let mut pool = db::init_pool();

    let conn = SqliteConnection::establish(&db::database_url()).unwrap();
    embedded_migrations::run_with_output(&conn, &mut std::io::stdout()).unwrap();

    let f = move || {
        if let Err(e) = instagram::update_image_paths(&mut pool) {
            log::error!("{}", e);
        }
    };
    thread::spawn(f.clone());
    let mut scheduler = Scheduler::new();
    scheduler.every(3.hours()).run(f);

    rocket::ignite()
        .manage(db::init_pool())
        .mount("/", routes![api::v1::dog::dog])
        .launch();
}
