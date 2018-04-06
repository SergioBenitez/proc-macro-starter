#![feature(proc_macro)]

#[macro_use] extern crate proc_macro_starter;
extern crate rocket;

#[derive(FromFormValue)]
pub enum Value {
    A,
    B,
    C,
    SomethingElse,
}

pub fn main() { }
