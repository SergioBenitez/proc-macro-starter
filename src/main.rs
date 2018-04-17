#![feature(proc_macro)]

#[macro_use] extern crate proc_macro_starter;
extern crate rocket;

use std::fmt;
use rocket::http::uri::Uri;

pub struct UriFormatter<'f, 'i: 'f> {
    prefixes: Vec<&'static str>,
    inner: &'f mut fmt::Formatter<'i>,
    previous: bool,
    fresh: bool
}

impl<'f, 'i: 'f> UriFormatter<'f, 'i> {
    fn write_raw(&mut self, s: &str) -> fmt::Result {
        if self.fresh && !self.prefixes.is_empty() {
            if self.previous {
                self.inner.write_str("&")?;
            }

            self.fresh = false;
            self.previous = true;

            for (i, prefix) in self.prefixes.iter().enumerate() {
                self.inner.write_str(prefix)?;
                if i < self.prefixes.len() - 1 {
                    self.inner.write_str(".")?;
                }
            }

            self.inner.write_str("=")?;
        }

        self.inner.write_str(s)
    }
    fn with_prefix<F>(&mut self, prefix: &'static str, f: F) -> fmt::Result
        where F: FnOnce(&mut Self) -> fmt::Result
    {
        self.fresh = true;
        self.prefixes.push(prefix);

        let result = f(self);

        self.prefixes.pop();
        result
    }
}

impl<'f, 'i: 'f> fmt::Write for UriFormatter<'f, 'i> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // println!("\nfmt::write_str({})", s);
        self.write_raw(s)
    }
}

pub trait _UriDisplay {
    fn fmt(&self, f: &mut UriFormatter) -> fmt::Result;
}

impl _UriDisplay for str {
    fn fmt(&self, f: &mut UriFormatter) -> fmt::Result {
        f.write_raw(&Uri::percent_encode(self))
    }
}

impl<'a> _UriDisplay for &'a str {
    fn fmt(&self, f: &mut UriFormatter) -> fmt::Result {
        _UriDisplay::fmt(*self, f)
    }
}

impl _UriDisplay for u8 {
    fn fmt(&self, f: &mut UriFormatter) -> fmt::Result {
        use fmt::Write;
        write!(f, "{}", self)
    }
}

impl<'a, T: _UriDisplay> _UriDisplay for &'a T {
    fn fmt(&self, f: &mut UriFormatter) -> fmt::Result {
        _UriDisplay::fmt(*self, f)
    }
}

impl<'a> fmt::Display for &'a _UriDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut uri_formatter = UriFormatter {
            prefixes: Vec::new(),
            inner: f,
            previous: false,
            fresh: true,
        };

        _UriDisplay::fmt(*self, &mut uri_formatter)
    }
}

#[derive(_UriDisplay)]
enum Test<'a> {
    Foo(&'a u8)
}

#[derive(_UriDisplay)]
pub struct Name<'a>(&'a str);

#[derive(_UriDisplay)]
pub struct X<'a, 'b> where 'b: 'a {
    a: &'a str,
    b: &'b str
}

#[derive(_UriDisplay)]
pub struct Animal {
    name: &'static str,
    color: &'static str
}

#[derive(_UriDisplay)]
pub struct Person {
    name: &'static str,
    age: u8,
    pet: Animal
}

#[derive(_UriDisplay)]
pub struct BigInt{
    int: u8
}

pub struct Complex(u8, u8);

impl _UriDisplay for Complex {
    fn fmt(&self, f: &mut UriFormatter) -> fmt::Result {
        use fmt::Write;
        write!(f, "{}+{}", self.0, self.1)
    }
}

#[derive(_UriDisplay)]
pub enum Shape {
    Rectangle(u8),
    Circle(u8),
    Sphere { radius: Complex, center: BigInt}
}

pub fn main() {

    let p = Animal{ name: "clifford", color: "red" };
    let e = Person { name: "emily", age: 5, pet : p };
    println!("{}", &e as &_UriDisplay);
    assert_eq!((&e as &_UriDisplay).to_string(), "name=emily&age=5&pet.name=clifford&pet.color=red");

    let c = Complex(1, 2);
    let s = Shape::Sphere { radius: c, center: BigInt { int: 3 } };
    println!("{}", (&s as &_UriDisplay));
    assert_eq!((&s as &_UriDisplay).to_string(), "radius=1+2&center.int=3");
}
