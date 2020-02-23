use diesel;
use diesel::prelude::*;
use meme_machine_database;
use meme_machine_database::models::*;
use meme_machine_database::*;

use std::io::stdin;

fn main() {
    use meme_machine_database::schema::rigs::dsl::*;

    let mut rig = String::new();

    println!("\nWhat is the title of the rig you want to delete?");
    stdin()
        .read_line(&mut rig)
        .expect("Please enter a title for the Rig");
    let rig = rig.trim();
    let pattern = format!("%{}%", rig);

    let connection = establish_connection();
    let matching_rigs = rigs
        .filter(active.eq(true))
        .load::<Rig>(&connection)
        .expect("Error loading rigs");
    let mut match_found = false;

    for test_rig in matching_rigs {
        if test_rig.title == rig {
            match_found = true;
            break;
        }
    }

    if match_found {
        //TODO replace .like with an exact match
        let num_deleted = diesel::delete(rigs.filter(title.like(pattern)))
            .execute(&connection)
            .expect("Error deleting rigs");

        if num_deleted > 0 {
            println!("\nDeleted {} rigs", num_deleted);
        } else {
            println!("\nOOPS: Could not find a rig named '{}'", rig);
        }
    } else {
        println!("\nOOPS: There are no rigs matching that name.");
    }
}
