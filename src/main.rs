#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate diesel;

pub mod schema;

use crate::db::DbConn;

#[database("sqlite_logs")]
struct SqliteConn(diesel::SqliteConnection);

#[get("/")]
fn index(conn: DbConn) -> String {
    match db::dogs::get_one(&conn) {
        // TODO load the pic & return that
        Ok(d) => d.path,
        Err(e) => format!("Error: {}", e)
    }
}

fn main() {
    dotenv::dotenv().expect("Failed to read .env file");
    rocket::ignite()
       .manage(db::init_pool())
       .mount("/", routes![index])
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
            Pool::new(manager).expect("db pool")
        }

        fn database_url() -> String {
            env::var("DATABASE_URL").expect("DATABASE_URL must be set")
        }

        pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<SqliteConnection>>);

        impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
            type Error = ();

            fn from_request(
                request: &'a Request<'r>,
            ) -> request::Outcome<DbConn, Self::Error> {
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
        use diesel::sql_types::BigInt;
        use diesel::prelude::*;
        use crate::db::DbConn;

        #[derive(Queryable, QueryableByName, Identifiable)]
        #[table_name = "dogs"]
        #[primary_key("path")]
        pub struct Dog {
            pub path: String,
        }

        pub fn get_one(conn: &SqliteConnection) -> QueryResult<Dog> {
            //dogs::table.first::<Dog>(&*conn)
            dogs::table.order::<SqlLiteral<BigInt>>(sql("RANDOM()")).first(&*conn)

            //let dog_vec = diesel::sql_query("SELECT * FROM dogs ORDER BY RANDOM() LIMIT 1").get_result(conn)?;
            //Ok(dog_vec[0])

        }
    }
}
