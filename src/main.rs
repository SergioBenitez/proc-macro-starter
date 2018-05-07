#![feature(proc_macro)]

extern crate proc_macro_starter;
extern crate rocket;
extern crate rocket_contrib;

use rocket_contrib::databases::diesel;
use proc_macro_starter::database;

#[database("my_sqlite_database")]
struct TempStorage(diesel::SqliteConnection);

// -------------- Invalid examples --------------
//#[database("my_sqlite_database")]
//struct TempStorage;
//
//#[database("my_sqlite_database")]
//enum TempStorage {Thing(i32)}
//
//#[database("my_sqlite_database")]
//struct TempStorage<T>(T);

pub fn main() { }
