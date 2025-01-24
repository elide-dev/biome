//! Extremely fast, lossless, and error tolerant JavaScript Parser.
//!
//! The parser uses an abstraction over non-whitespace tokens.
//! This allows us to losslessly or lossly parse code without requiring explicit handling of whitespace.
//! The parser yields events, not an AST, the events are resolved into untyped syntax nodes, which can then
//! be casted into a typed AST.
//!
//! The parser is able to produce a valid AST from **any** source code.
//! Erroneous productions are wrapped into `ERROR` syntax nodes, the original source code
//! is completely represented in the final syntax nodes.
//!
//! You probably do not want to use the parser struct, unless you want to parse fragments of Js source code or make your own productions.
//! Instead use functions such as [parse_script], and [parse_module] which offer abstracted versions for parsing.
//!
//! For more finer control, use [parse](crate::parse::parse()) or [parse_js_with_cache],
//!
//! Notable features of the parser are:
//! - Extremely fast parsing and lexing through the extremely fast lexer.
//! - Ability to do Lossy or Lossless parsing on demand without explicit whitespace handling.
//! - Customizable, able to parse any fragments of JS code at your discretion.
//! - Completely error tolerant, able to produce an AST from any source code.
//! - Zero cost for converting untyped nodes to a typed AST.
//! - Ability to go from AST to SyntaxNodes to SyntaxTokens to source code and back very easily with nearly zero cost.
//! - Very easy tree traversal through [`SyntaxNode`](biome_rowan::SyntaxNode).
//! - Descriptive errors with multiple labels and notes.
//! - Very cheap cloning, cloning an ast node or syntax node is the cost of adding a reference to an Rc.
//! - Cheap incremental reparsing of changed text.
//!
//! The crate further includes utilities such as:
//! - ANSI syntax highlighting of nodes or text through `lexer`.
//!
//! It is inspired by the rust analyzer parser but adapted for JavaScript.
//!
//! # Syntax Nodes vs AST Nodes
//! The crate relies on a concept of untyped [biome_js_syntax::JsSyntaxNode]s vs typed [biome_rowan::AstNode]s.
//! Syntax nodes represent the syntax tree in an untyped way. They represent a location in an immutable
//! tree with two pointers. The syntax tree is composed of [biome_js_syntax::JsSyntaxNode]s and [biome_js_syntax::JsSyntaxToken]s in a nested
//! tree structure. Each node can have parents, siblings, children, descendants, etc.
//!
//! [biome_rowan::AstNode]s represent a typed version of a syntax node. They have the same exact representation as syntax nodes
//! therefore a conversion between either has zero runtime cost. Every piece of data of an ast node is optional,
//! this is due to the fact that the parser is completely error tolerant.
//!
//! Each representation has its advantages:
//!
//! ### SyntaxNodes
//! - Very simple traversing of the syntax tree through functions on them.
//! - Easily able to convert to underlying text, range, or tokens.
//! - Contain all whitespace bound to the underlying production (in the case of lossless parsing).
//! - Can be easily converted into its typed representation with zero cost.
//! - Can be turned into a pretty representation with fmt debug.
//!
//! ### AST Nodes
//! - Easy access to properties of the underlying production.
//! - Zero cost conversion to a syntax node.
//!
//! In conclusion, the use of both representations means we are not constrained to acting through
//! typed nodes. Which makes traversal hard and you often have to resort to autogenerated visitor patterns.
//! AST nodes are simply a way to easily access subproperties of a syntax node.event;

mod parser;
#[macro_use]
mod lexer;
pub mod options;
mod parse;
mod prelude;
mod rewrite;
mod span;
mod state;
pub mod syntax;
mod token_source;

use crate::prelude::*;
pub(crate) use crate::ParsedSyntax::{Absent, Present};
pub use crate::{
    lexer::{JsLexContext, JsReLexContext},
    options::JsParserOptions,
    parse::*,
};
use biome_js_factory::JsSyntaxFactory;
use biome_js_syntax::{JsSyntaxKind, LanguageVariant};
use biome_parser::tree_sink::LosslessTreeSink;
pub(crate) use parser::{JsParser, ParseRecoveryTokenSet};
pub(crate) use state::{JsParserState, StrictMode};
use std::fmt::Debug;

pub enum JsSyntaxFeature {
    #[doc(alias = "LooseMode")]
    SloppyMode,
    StrictMode,
    TypeScript,
    Jsx,
}

impl SyntaxFeature for JsSyntaxFeature {
    type Parser<'source> = JsParser<'source>;

    fn is_supported(&self, p: &JsParser) -> bool {
        match self {
            JsSyntaxFeature::SloppyMode => p.state().strict().is_none(),
            JsSyntaxFeature::StrictMode => p.state().strict().is_some(),
            JsSyntaxFeature::TypeScript => p.source_type().language().is_typescript(),
            JsSyntaxFeature::Jsx => p.source_type().variant() == LanguageVariant::Jsx,
        }
    }
}

pub(crate) type JsLosslessTreeSink<'source> =
    LosslessTreeSink<'source, JsLanguage, JsSyntaxFactory>;
