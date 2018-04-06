#![allow(dead_code)]

use syn::token;
use syn::synom::Synom;
use syn::buffer::{Cursor, TokenBuffer};

use proc_macro::{TokenStream, Span, Diagnostic};

pub use proc_macro2::Delimiter;

pub type Result<T> = ::std::result::Result<T, Diagnostic>;

pub enum Seperator {
    Comma,
    Pipe,
    Semi,
}

pub struct Parser {
    buffer: Box<TokenBuffer>,
    cursor: Cursor<'static>,
}

impl Parser {
    pub fn new(tokens: TokenStream) -> Parser {
        let buffer = Box::new(TokenBuffer::new(tokens.into()));
        let cursor = unsafe {
            let buffer: &'static TokenBuffer = ::std::mem::transmute(&*buffer);
            buffer.begin()
        };

        Parser {
            buffer: buffer,
            cursor: cursor,
        }
    }

    pub fn current_span(&self) -> Span {
        self.cursor.token_tree()
            .map(|_| self.cursor.span().unstable())
            .unwrap_or_else(|| Span::call_site())
    }

    pub fn parse<T: Synom>(&mut self) -> Result<T> {
        let (val, cursor) = T::parse(self.cursor)
            .map_err(|e| {
                let expected = match T::description() {
                    Some(desc) => desc,
                    None => unsafe { ::std::intrinsics::type_name::<T>() }
                };

                self.current_span().error(format!("{}: expected {}", e, expected))
            })?;

        self.cursor = cursor;
        Ok(val)
    }

    pub fn eat<T: Synom>(&mut self) -> bool {
        self.parse::<T>().is_ok()
    }

    pub fn parse_group<F, T>(&mut self, delim: Delimiter, f: F) -> Result<T>
        where F: FnOnce(&mut Parser) -> Result<T>
    {
        if let Some((group_cursor, _, next_cursor)) = self.cursor.group(delim) {
            self.cursor = group_cursor;
            let result = f(self);
            self.cursor = next_cursor;
            result
        } else {
            let expected = match delim {
                Delimiter::Brace => "curly braced group",
                Delimiter::Bracket => "square bracketed group",
                Delimiter::Parenthesis => "parenthesized group",
                Delimiter::None => "invisible group"
            };

            Err(self.current_span()
                .error(format!("parse error: expected {}", expected)))
        }
    }

    pub fn parse_sep<F, T>(&mut self, sep: Seperator, mut f: F) -> Result<Vec<T>>
        where F: FnMut(&mut Parser) -> Result<T>
    {
        let mut output = vec![];
        while !self.is_eof() {
            output.push(f(self)?);
            let have_sep = match sep {
                Seperator::Comma => self.eat::<token::Comma>(),
                Seperator::Pipe => self.eat::<token::Or>(),
                Seperator::Semi => self.eat::<token::Semi>(),
            };

            if !have_sep {
                break;
            }
        }

        Ok(output)
    }

    pub fn eof(&self) -> Result<()> {
        if !self.cursor.eof() {
            let diag = self.current_span()
                .error("trailing characters; expected eof");

            return Err(diag);
        }

        Ok(())
    }

    fn is_eof(&self) -> bool {
        self.eof().is_ok()
    }
}
