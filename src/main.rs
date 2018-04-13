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

#[derive(UriDisplay)]
pub struct BigInt(u32);

#[derive(UriDisplay)]
pub enum Shape {
	Rectangle(u32),
	Circle(u32),
	Sphere { center: u32, radius: BigInt }
}

pub fn main() {
	let p = Person { name: "john smith", age: 5 };
	use rocket::http::uri::UriDisplay;
	println!("{}", (&p as &UriDisplay).to_string());
	assert_eq!((&p as &UriDisplay).to_string(), "name=john%20smith&age=5");

	let q = BigInt(6);
	let r = Shape::Sphere { center: 3, radius: q };
	println!("{}", (&r as &UriDisplay).to_string());

	let z = q; // fails because q moved.
}
