#![feature(proc_macro)]

#[macro_use] extern crate proc_macro_starter;
extern crate rocket;
extern crate rocket_contrib;

use rocket_contrib::databases::{diesel, r2d2, postgres, Poolable};

#[derive(FromFormValue)]
pub enum Value {
    A,
    B,
    C,
    SomethingElse,
}

#[derive(DbConn)]
#[database = "my_sqlite_database"]
struct TempStoragePool(r2d2::Pool<<diesel::SqliteConnection as Poolable>::Manager>);
//struct TempStoragePool;

#[derive(DbConn)]
#[database = "primary_database"]
struct PrimaryDatabase(r2d2::Pool<<postgres::Connection as Poolable>::Manager>);
//struct PrimaryDatabase

pub fn main() { }
