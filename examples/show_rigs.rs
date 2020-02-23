use diesel::prelude::*;
use meme_machine_database::models::*;
use meme_machine_database::*;
use schema::rigs::dsl::*;

fn main() {
    let connection = establish_connection();
    let results = rigs
        .filter(active.eq(true))
        //        .limit(5)
        .load::<Rig>(&connection)
        .expect("Error loading rigs");

    println!("\nDisplaying {} rigs\n============\n", results.len());
    for rig in results {
        println!("---- {} ----", rig.title);
        println!("{}", rig.body);
        println!("-----------\n");
    }
}
