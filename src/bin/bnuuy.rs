use bnuuy::{api, bnuuy_config, db, image_sources::ImageSource, logger};
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
    logger::setup_logging(log::LevelFilter::Debug).unwrap();

    let pool = db::init_pool();

    let conn = SqliteConnection::establish(&db::database_url()).unwrap();
    embedded_migrations::run_with_output(&conn, &mut std::io::stdout()).unwrap();

    let config = bnuuy_config::read_config().unwrap();
    for source in config.sources {
        let mut p = pool.clone();
        let f = move || {
            if let Err(e) = source.update_image_paths(&mut p) {
                log::error!("{}", e);
            }
        };
        thread::spawn(f.clone());
        let mut scheduler = Scheduler::new();
        // TODO allow cadence to be set?
        scheduler.every(3.hours()).run(f);
    }

    rocket::ignite()
        .manage(db::init_pool())
        .mount("/", routes![api::v1::dog::dog])
        .launch();
}
