#![feature(proc_macro)]

extern crate proc_macro_starter;
extern crate rocket;
extern crate rocket_contrib;

use rocket_contrib::databases::diesel;
use proc_macro_starter::database;

#[database("my_sqlite_database")]
struct TempStorage(diesel::SqliteConnection);

pub fn main() { }
