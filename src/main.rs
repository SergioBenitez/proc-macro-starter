#![feature(proc_macro)]

#[macro_use] extern crate proc_macro_starter;
extern crate rocket;

#[derive(UriDisplay)]
pub struct Person {
	name: &'static str,
	age: u8,
}

#[derive(UriDisplay)]
pub struct Animal(Person);

pub fn main() {
	let p = Person { name: "john smith", age: 5 };
	use rocket::http::uri::UriDisplay;
	println!("{}", (&p as &UriDisplay).to_string());
	assert_eq!((&p as &UriDisplay).to_string(), "name=john%20smith&age=5");

	let q = Animal(p);
	println!("{}", (&q as &UriDisplay).to_string());
	assert_eq!((&q as &UriDisplay).to_string(), "name=john%20smith&age=5");
}
