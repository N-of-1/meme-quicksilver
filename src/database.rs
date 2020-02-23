#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod models;
pub mod schema;

use self::models::{NewRig, Rig};
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub fn create_rig<'a>(conn: &PgConnection, title: &'a str, body: &'a str) -> Rig {
    use schema::rigs;

    let new_rig = NewRig {
        title: title,
        body: body,
    };

    diesel::insert_into(rigs::table)
        .values(&new_rig)
        .get_result(conn)
        .expect("Error saving new rig")
}
