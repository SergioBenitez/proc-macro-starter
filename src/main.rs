#![feature(proc_macro)]

#[macro_use] extern crate proc_macro_starter;
extern crate rocket;
extern crate rocket_contrib;

use rocket_contrib::databases::diesel;

#[derive(DbConn)]
#[database = "my_sqlite_database"]
struct TempStorage(diesel::SqliteConnection);

pub fn main() { }
