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
