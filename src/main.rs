#![feature(proc_macro)]

#[macro_use] extern crate proc_macro_starter;
extern crate smallvec;
extern crate rocket;

use std::fmt;
use smallvec::SmallVec;
use rocket::http::uri::Uri;

pub struct Formatter<'f, 'i: 'f> {
    prefixes: SmallVec<[&'static str; 3]>,
    inner: &'f mut fmt::Formatter<'i>,
    previous: bool,
    fresh: bool
}

impl<'f, 'i: 'f> Formatter<'f, 'i> {
    pub fn write_raw<S: AsRef<str>>(&mut self, s: S) -> fmt::Result {
        let s = s.as_ref();
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

    fn with_prefix<F>(&mut self, prefix: &str, f: F) -> fmt::Result
        where F: FnOnce(&mut Self) -> fmt::Result
    {
        self.fresh = true;

        // TODO: PROOF OF CORRECTNESS.
        let prefix: &'static str = unsafe { ::std::mem::transmute(prefix) };
        self.prefixes.push(prefix);

        let result = f(self);

        self.prefixes.pop();
        result
    }

    pub fn write_seq_value<T: _UriDisplay>(&mut self, value: T) -> fmt::Result {
        self.fresh = true;
        self.write_value(value)
    }

    pub fn write_named_seq_value<T: _UriDisplay>(&mut self, name: &str, value: T) -> fmt::Result {
        self.write_named_value(name, value)
    }

    #[inline]
    pub fn write_named_value<T: _UriDisplay>(&mut self, name: &str, value: T) -> fmt::Result {
        self.with_prefix(name, |f| f.write_value(value))
    }

    #[inline]
    pub fn write_value<T: _UriDisplay>(&mut self, value: T) -> fmt::Result {
        _UriDisplay::fmt(&value, self)
    }
}

impl<'f, 'i: 'f> fmt::Write for Formatter<'f, 'i> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // println!("\nfmt::write_str({})", s);
        self.write_raw(s)
    }
}

pub trait _UriDisplay {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result;
}

impl _UriDisplay for str {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_raw(&Uri::percent_encode(self))
    }
}

impl<'a> _UriDisplay for &'a str {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        _UriDisplay::fmt(*self, f)
    }
}

impl _UriDisplay for u8 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use fmt::Write;
        write!(f, "{}", self)
    }
}

impl<'a, T: _UriDisplay> _UriDisplay for &'a T {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        _UriDisplay::fmt(*self, f)
    }
}

impl<'a> fmt::Display for &'a _UriDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut formatter = Formatter {
            prefixes: SmallVec::new(),
            inner: f,
            previous: false,
            fresh: true,
        };

        _UriDisplay::fmt(*self, &mut formatter)
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
pub struct BigInt(u8);

pub struct Complex(u8, u8);

impl _UriDisplay for Complex {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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

#[derive(_UriDisplay)]
pub struct Baz {
    q: Qux
}

pub struct Qux(FooBar, FooBar);

#[derive(_UriDisplay)]
pub struct FooBar {
    x: u8,
    y: u8
}

impl _UriDisplay for Qux {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_seq_value(&self.0)?;
        f.write_seq_value(&self.1)?;
        Ok(())
    }
}

#[derive(_UriDisplay)]
pub struct Generic<T> {
    x: T
}

#[derive(_UriDisplay)]
pub struct NestedGeneric<T> where T: _UriDisplay {
    y: Generic<T>
}

// #[derive(_UriDisplay)]
// pub enum TestBad {
//     First { b: Bad }
// }

// #[derive(_UriDisplay)]
// pub enum TestBad {
//     First { b: Bad }
// }


pub fn main() {
    let p = Animal{ name: "clifford", color: "red" };
    let e = Person { name: "emily", age: 5, pet : p };
    println!("{}", &e as &_UriDisplay);
    assert_eq!((&e as &_UriDisplay).to_string(), "name=emily&age=5&pet.name=clifford&pet.color=red");

    let c = Complex(1, 2);
    let s = Shape::Sphere { radius: c, center: BigInt(3) };
    println!("{}", (&s as &_UriDisplay));
    assert_eq!((&s as &_UriDisplay).to_string(), "radius=1+2&center=3");

    let a = FooBar { x: 1, y: 2 };
    let b = FooBar { x: 8, y: 9 };
    let q = Qux(a, b);
    let z = Baz { q: q };
    println!("{}", &z as &_UriDisplay);
    assert_eq!((&z as &_UriDisplay).to_string(), "q.x=1&q.y=2&q.x=8&q.y=9");
}
