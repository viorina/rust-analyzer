//! Lexing, bridging to ra_parser (which does the actual parsing) and
//! incremental reparsing.

mod lexer;
mod text_token_source;
mod text_tree_sink;
mod reparsing;

use crate::{
    SyntaxError,
    syntax_node::GreenNode,
};

pub use self::lexer::{tokenize, classify_literal, Token};

pub(crate) use self::reparsing::incremental_reparse;

pub(crate) fn parse_text(text: &str) -> (GreenNode, Vec<SyntaxError>) {
    let tokens = tokenize(&text);
    let token_source = text_token_source::TextTokenSource::new(text, &tokens);
    let mut tree_sink = text_tree_sink::TextTreeSink::new(text, &tokens);
    ra_parser::parse(&token_source, &mut tree_sink);
    tree_sink.finish()
}