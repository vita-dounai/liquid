// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::iter::FromIterator;
use proc_macro2::{
    token_stream::IntoIter as TokenIter, Delimiter, Group, Ident, Literal, Punct,
    Spacing, Span, TokenStream as TokenStream2, TokenTree,
};

pub struct SyntaxError {
    message: String,
    span: Span,
}

impl SyntaxError {
    pub fn new(message: String, token: &TokenTree) -> Self {
        Self {
            message,
            span: token.span(),
        }
    }

    pub fn into_compile_error(self) -> TokenStream2 {
        let token_stream = [
            TokenTree::Ident(Ident::new("compile_error", self.span)),
            TokenTree::Punct({
                let mut punct = Punct::new('!', Spacing::Alone);
                punct.set_span(self.span);
                punct
            }),
            TokenTree::Group({
                let mut group = Group::new(
                    Delimiter::Brace,
                    TokenStream2::from_iter(
                        [TokenTree::Literal({
                            let mut string = Literal::string(&self.message);
                            string.set_span(self.span);
                            string
                        })]
                        .to_vec(),
                    ),
                );
                group.set_span(self.span);
                group
            }),
        ];
        TokenStream2::from_iter(token_stream.to_vec())
    }
}

pub type Result<T> = core::result::Result<T, SyntaxError>;

fn next_token(iter: &mut TokenIter) -> Result<TokenTree> {
    iter.next().ok_or_else(|| SyntaxError {
        message: "unexpected end of input".to_owned(),
        span: Span::call_site(),
    })
}

pub fn expect_ident(iter: &mut TokenIter) -> Result<Ident> {
    match next_token(iter)? {
        TokenTree::Ident(ident) => Ok(ident),
        other => Err(SyntaxError::new("expected ident".to_owned(), &other)),
    }
}

pub fn expect_keyword(iter: &mut TokenIter, keyword: &str) -> Result<()> {
    let token = next_token(iter)?;
    if let TokenTree::Ident(ident) = &token {
        if ident == keyword {
            return Ok(());
        }
    }
    Err(SyntaxError::new(format!("expected `{}`", keyword), &token))
}

pub fn expect_integer(iter: &mut TokenIter) -> Result<u64> {
    let token = next_token(iter)?;
    if let TokenTree::Literal(literal) = &token {
        if let Ok(integer) = literal.to_string().parse::<u64>() {
            return Ok(integer);
        }
    }

    Err(SyntaxError::new(
        "expected unsuffixed integer literal".to_owned(),
        &token,
    ))
}

pub fn expect_punct(iter: &mut TokenIter, ch: char) -> Result<()> {
    let token = next_token(iter)?;
    if let TokenTree::Punct(punct) = &token {
        if punct.as_char() == ch {
            return Ok(());
        }
    }

    Err(SyntaxError::new(format!("expected `{}`", ch), &token))
}

pub fn expect_optional_punct(iter: &mut TokenIter, ch: char) -> Result<bool> {
    let present = match iter.clone().next() {
        Some(TokenTree::Punct(_)) => {
            expect_punct(iter, ch)?;
            true
        }
        _ => false,
    };
    Ok(present)
}

pub fn expect_body(iter: &mut TokenIter) -> Result<TokenStream2> {
    let token = next_token(iter)?;
    if let TokenTree::Group(group) = &token {
        if group.delimiter() == Delimiter::Brace {
            return Ok(group.stream());
        }
    }
    Err(SyntaxError::new("expected curly braces".to_owned(), &token))
}

pub fn expect_end(iter: &mut TokenIter) -> Result<()> {
    if let Some(token) = iter.next() {
        return Err(SyntaxError::new("unexpected token".to_owned(), &token));
    }
    Ok(())
}