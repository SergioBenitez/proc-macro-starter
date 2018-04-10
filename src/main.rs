#![feature(proc_macro)]

#[macro_use] extern crate proc_macro_starter;
extern crate rocket;
extern crate rocket_contrib;

use rocket_contrib::databases::{diesel, r2d2, Poolable};

#[derive(FromFormValue)]
pub enum Value {
    A,
    B,
    C,
    SomethingElse,
}

#[derive(DbConn)]
#[connection_name = "my_sqlite_database"]
struct TempStoragePool(r2d2::Pool<<diesel::SqliteConnection as Poolable>::Manager>);
//struct TempStoragePool;

pub fn main() { }
