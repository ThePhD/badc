use core::panic;
use std::borrow::Borrow;
use std::fs::File;
use std::io::{Read, Stdout};
use std::path::PathBuf;

use lex::Token;
use logos::Logos;

pub mod ast;
pub mod lex;

pub enum ProgramSource {
	Path(PathBuf),
	File(File),
	Stdin(Stdin),
}

pub enum ProgramSink {
	Path(PathBuf),
	File(File),
	Stdout(Stdout),
}

pub struct CompilationConfiguration {
	pub input: ProgramSource,
	pub print_tokens: bool,
	pub print_ast: bool,
	pub output: ProgramSink,
	pub print_tokens_output: ProgramSink,
	pub print_ast_output: ProgramSink,
}

#[derive(Debug)]
pub enum Token {
	LeftParen,
	RightParen,
	LeftBrace,
	RightBrace,
	Semicolon,
	SingleQuote,
	Codepoint(char),
	Identifier(String),
}

#[derive(Debug)]
pub struct TokenList {
	pub tokens: Vec<lex::Token>,
	pub context: ast::Context,
}

impl TokenList {
	pub fn new(path: PathBuf, source: String) -> TokenList {
		Self {
			tokens: Vec::new(),
			context: ast::Context::new(path, source)
		}
	}
}

#[derive(Debug)]
pub struct SyntaxTree {}

pub fn get_source_text(source: ProgramSource) -> (PathBuf, String) {
	let mut source_text: String = String::new();
	let mut source_path: PathBuf = PathBuf::new();
	match source {
		ProgramSource::Path(pathbuf) => { source_path = pathbuf.clone(); source_text = std::fs::read_to_string(pathbuf.as_path()).unwrap() },
		ProgramSource::File(mut file) => { source_path = "<file?handle?>".into(); file.read_to_string(&mut source_text); },
		ProgramSource::Stdin(mut stdin) => { source_path = "<stdin>".into(); stdin.read_to_string(&mut source_text); }
	};
	(source_path, source_text)
}

pub fn lex(source_path: PathBuf, source_text: String, config: &CompilationConfiguration) -> TokenList {
	let mut list: TokenList = TokenList::new(source_path, source_text);
	let logos_lex = Token::lexer(list.context.source()).spanned();
	for tok_and_range in logos_lex {
		match tok_and_range {
			(Token::Error, _range) => panic!("unrecognized sequence of text during scanning/lexing!"),
			(tok , _range) => list.tokens.push(tok)
		}
	};
	list
}

pub fn parse(
	_token_stream: TokenList,
	_config: &CompilationConfiguration,
) -> SyntaxTree {
	SyntaxTree {}
}

pub fn compile(source: ProgramSource, config: &CompilationConfiguration) -> SyntaxTree {
	let (source_path, source_text) = get_source_text(source);
	let lex: TokenList = lex(source_path, source_text, config);
	let tree: SyntaxTree = parse(lex, config);
	tree
}
