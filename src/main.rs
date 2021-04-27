#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;

pub mod schema;

use crate::db::{dogs::Dog, DbConn};
use base64;
use clokwerk::{Scheduler, TimeUnits};
use log;
use rocket::{
    http::Status,
    request::Request,
    response::{content, Responder, Response},
};
use std::fs;
use std::io::prelude::Read;
use walkdir::WalkDir;

use std::{error::Error, result::Result as StdResult};
type Result<T> = StdResult<T, Box<dyn Error>>;

const INSTA_USERNAME: &str = "MY-INSTA-USERNAME";
const INSTA_PASSWORD: &str = "MY-INSTA-PASSWORD";
static INSTA_PAGES: &'static [&str] = &[
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
const IMAGE_FOLDER: &str = "instagram-scraper";

#[derive(Debug)]
pub struct RespErr(Status);
type Resp<T> = StdResult<T, RespErr>;

impl<'r> Responder<'r> for RespErr {
    fn respond_to(self, req: &Request) -> StdResult<Response<'r>, Status> {
        self.0.respond_to(req)
    }
}

/// Allow error handling with `?` for 500 errors.
impl From<diesel::result::Error> for RespErr {
    fn from(error: diesel::result::Error) -> Self {
        log::error!("{}", error);
        RespErr(Status::ServiceUnavailable)
    }
}

impl From<std::io::Error> for RespErr {
    fn from(error: std::io::Error) -> Self {
        log::error!("{}", error);
        RespErr(Status::ServiceUnavailable)
    }
}

#[get("/dog")]
fn dog(conn: DbConn) -> Resp<content::Html<String>> {
    let path = db::dogs::get_random(&conn)?.path;
    log::debug!("Pulling image from {}", path);
    let b64_image = image_as_b64(path)?;

    Ok(content::Html(format!(
        "<html><center>
        <img src=\"data:image/jpeg;base64, {}\"/></center>
        <script>setTimeout(function(){{ window.location.reload(1); }}, 10000);
        </script></html>",
        b64_image
    )))
}

fn image_as_b64(path: String) -> Resp<String> {
    let mut f = fs::File::open(path)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    Ok(base64::encode(buffer))
}

fn update_image_paths(pool: &mut db::Pool) -> Result<()> {
    log::debug!("Adding new images & wiping old images..");

    // TODO
    // "instagram-scraper --destination ./cache/instagram --retain-username --media-metadata --media-types image --latest --login-user {} --login-pass {} --maximum 100 {page}"

    let conn = db::DbConn(pool.get()?);

    let mut images = Vec::new();
    for entry in WalkDir::new(IMAGE_FOLDER) {
        let e = match entry {
            Err(err) => {
                log::warn!("{}", err);
                continue;
            }
            Ok(e) => e,
        };
        if e.file_type().is_file() {
            let path = e.path().to_str().unwrap().to_string();
            if path.ends_with("jpg") || path.ends_with("jpeg") {
                let created = fs::metadata(path.clone()).unwrap().created().unwrap();
                images.push((path, created));
            }
        }
    }

    // Reverse sort by date
    images.sort_by(|(_, time_a), (_, time_b)| time_b.partial_cmp(time_a).unwrap());

    // Retain the N most recent
    images.truncate(1000);

    let dogs = images
        .iter()
        .map(|(path, _time)| Dog { path: path.clone() })
        .collect();

    db::dogs::delete_all(&conn)?;
    db::dogs::insert_many(dogs, &conn)?;

    log::debug!("Done adding new paths");
    Ok(())
}

fn main() {
    dotenv::dotenv().expect("Failed to read .env file");
    logger::setup_logging(log::LevelFilter::Debug).expect("failed to initialize logging");

    let mut pool = db::init_pool();
    let f = move || {
        if let Err(e) = update_image_paths(&mut pool) {
            log::error!("{}", e);
        }
    };
    let mut scheduler = Scheduler::new();
    scheduler.every(3.hours()).run(f);

    rocket::ignite()
        .manage(db::init_pool())
        .mount("/", routes![dog])
        .launch();
}

pub mod db {
    use diesel::sqlite::SqliteConnection;

    use r2d2_diesel::ConnectionManager;
    use rocket::{
        http::Status,
        request::{self, FromRequest},
        Outcome, Request, State,
    };
    use std::{env, ops::Deref};

    pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

    pub fn init_pool() -> Pool {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url());
        Pool::new(manager).expect("Could not initialize db pool")
    }

    fn database_url() -> String {
        env::var("DATABASE_URL").expect("DATABASE_URL must be set")
    }

    pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<SqliteConnection>>);

    impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
        type Error = ();

        fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, Self::Error> {
            let pool = request.guard::<State<Pool>>()?;
            match pool.get() {
                Ok(conn) => Outcome::Success(DbConn(conn)),
                Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
            }
        }
    }

    impl Deref for DbConn {
        type Target = SqliteConnection;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    pub mod dogs {
        use crate::schema::dogs;
        use diesel;
        use diesel::dsl::sql;
        use diesel::expression::SqlLiteral;
        use diesel::prelude::*;
        use diesel::sql_types::BigInt;

        #[derive(Queryable, QueryableByName, Identifiable, Insertable)]
        #[table_name = "dogs"]
        #[primary_key("path")]
        pub struct Dog {
            pub path: String,
        }

        pub fn get_random(conn: &SqliteConnection) -> QueryResult<Dog> {
            dogs::table
                .order::<SqlLiteral<BigInt>>(sql("RANDOM()"))
                .first(&*conn)
        }

        pub fn delete_all(conn: &SqliteConnection) -> QueryResult<usize> {
            diesel::delete(dogs::table).execute(&*conn)
        }

        pub fn insert(dog: Dog, connection: &SqliteConnection) -> QueryResult<usize> {
            insert_many(vec![dog], connection)
        }

        pub fn insert_many(dogs: Vec<Dog>, connection: &SqliteConnection) -> QueryResult<usize> {
            diesel::insert_into(dogs::table)
                .values(dogs)
                .execute(connection)
        }
    }
}

pub mod logger {

    use std::io;

    pub fn setup_logging(verbosity: log::LevelFilter) -> Result<(), fern::InitError> {
        let base_config = fern::Dispatch::new().level(verbosity);

        let stdout_config = fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.target(),
                    record.level(),
                    message
                ))
            })
            .chain(io::stdout());

        base_config.chain(stdout_config).apply()?;

        Ok(())
    }
}
