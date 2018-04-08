#![feature(proc_macro)]

#[macro_use] extern crate proc_macro_starter;
extern crate rocket;

#[derive(UriDisplay)]
pub struct Person {
	name: &'static str,
	age: u8,
}

pub fn main() {
	let p = Person { name: "john smith", age: 5 };
	use rocket::http::uri::UriDisplay;
	println!("{}", &p as &UriDisplay);
}
