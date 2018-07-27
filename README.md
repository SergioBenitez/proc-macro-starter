proc-macro-starter
==================

This repository contains an implementation of a derive for the [`FromFormValue`]
Rocket trait. The implementation is in `lib.rs`, and a use of the macro is in
`main.rs`. To see what the macro invocation expands to, run `cargo expand --bin
main`. You'll need to `cargo install cargo-expand` first.

## Dependencies

The implementation depends on the following libraries:

  * [`proc_macro`](https://doc.rust-lang.org/nightly/proc_macro/index.html)

    This is the Rust standard library crate for procedural macros.

  * [`quote`](https://docs.rs/quote)

    The quasi-quoting crate providing the `quote!` and `quote_spanned!` macros.
    These macros take in Rust tokens and interpolated variables (`#variables`)
    and produce a `TokenStream`.
  
  * [`syn`](https://docs.rs/syn)

    Parser from `TokenStream` to ASTs.

  * [`proc_macro2`](https://docs.rs/proc-macro2)

    Crate that provide types that wrap the unstable `proc_macro` types,
    providing a pseudo-stable shim. `syn` exposes types from `proc_macro2`. This
    crate depends on it solely so that the "nightly" feature is enabled for
    `proc_macro2` when depended on by `syn` so that the exposed typed from `syn`
    have all of the unstable behavior.

## Structure

The entry point to a derive based procedural macro is annotated with
`#[proc_macro_derive(TraitName)]`. This is the `derive_from_form_value`
function. It must have the signature `TokenStream -> TokenStream`.

This function calls `real_derive_from_form_value` which returns the
`TokenStream` of the implementation if code generation succeeded or a
[`proc_macro::Diagnostic`] if it failed. The `PResult<T>` type is an alias for
`Result<T, Diagnostic>`.

To perform the actual code generation, the implementation first parses the
`TokenStream` into an AST of type `DeriveInput`, inspects it, then extracts the
information it needs for quasi-quoting. It then generates the implementation
using the extracted information and the `quote!` macro.

[`proc_macro::Diagnostic`]: https://doc.rust-lang.org/nightly/proc_macro/struct.Diagnostic.html
[`FromFormValue`]: https://api.rocket.rs/rocket/request/trait.FromFormValue.html

## `ext`, `spanned`, and `parser`

The `ext` module contains [extension traits] for `syn` types. These effectively
bring in missing useful functionality to external types.

The `spanned` module implements the `Spanned` trait, making available the
`span()` method, for almost every `syn` type. `syn` itself provides [such a
trait]. The difference is that this implementation returns a `proc_macro::Span`
as opposed to a `proc_macro2::Span`, so we can work directly with the unstable
APIs (in particular, diagnostic APIs) after a call to `span()`.

The `parser` module contains a `Parser` that can be used to parse arbitrary
`syn` items from a `TokenStream`. This is useful to parse things like the
contents of arbitrary attributes. A use might look like:

```rust
// Construct the parser from the tokens inside an attribute.
let mut parser = Parser::new(attr.tts.clone().into());

// Parse something that looks like: (a = 12, bcd = "hi") into a `Vec<(ident, lit)>`.
let parsed = parser.parse_group(Delimiter::Parenthesis, |parser| {
    parser.parse_sep(Seperator::Comma, |parser| {
        let ident: Ident = parser.parse()?;
        parser.parse::<token::Eq>()?;
        let value: Lit = parser.parse()?;

        Ok((ident, value))
    })
}).map_err(|_| attr.span().error(BAD_ATTR))?;
```

[such a trait]: https://docs.rs/syn/0.13.1/syn/spanned/trait.Spanned.html
[extension traits]: http://xion.io/post/code/rust-extension-traits.html

## Development

I like to use `cargo watch -x check` (`cargo install cargo-watch`) to
continuously run `cargo check` while I'm working on some code. You can also set
up your editor to do this automatically. Use `cargo check` instead of `cargo
build` while developing; it's much faster.

To see the output of your procedural macro, modify `main.rs` to use it in some
way, then run `cargo expand --bin main` to see what the expanded version looks
like. You'll need `rustfmt` and `Pygments` installed:

  * `rustup component add rustfmt-preview`
  * `pip install Pygments`

Diagnostics are particular important. To see how they work, try creating invalid
macro input, such as adding some generics to the structure, and seeing the
resulting error messages emitted by the procedural macro.

To merge this into Rocket, you'll need to write some positive and negative unit
tests. Don't worry about this at first. If you're curious to see what these look
like, see the [existing codegen test suite].

[existing codegen test suite]: https://github.com/SergioBenitez/Rocket/tree/master/codegen/tests
