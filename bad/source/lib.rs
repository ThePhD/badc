use core::panic;
use std::fs::File;
use std::io::{Read, Stdout};
use std::path::PathBuf;

use lex::TokenName;
use lex::Token;
use logos::Logos;

pub mod ast;
pub mod lex;
pub mod context;

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
	pub context: context::Context,
}

impl TokenList {
	pub fn new(path: PathBuf, source: String) -> TokenList {
		Self {
			tokens: Vec::new(),
			context: context::Context::new(path, source)
		}
	}
}

#[derive(Debug)]
pub struct SyntaxTree {}

pub fn get_source_text(source: ProgramSource) -> (PathBuf, String) {
	let mut source_text: String = String::new();
	let source_path: PathBuf;
	match source {
		ProgramSource::Path(pathbuf) => { source_path = pathbuf.clone(); source_text = std::fs::read_to_string(pathbuf.as_path()).unwrap() },
		ProgramSource::File(mut file) => { source_path = "<file?handle?>".into(); file.read_to_string(&mut source_text).unwrap(); },
		ProgramSource::Stdin(mut stdin) => { source_path = "<stdin>".into(); stdin.read_to_string(&mut source_text).unwrap(); }
	};
	(source_path, source_text)
}

pub fn lex(input_path: PathBuf, input_source: String, _config: &CompilationConfiguration) -> TokenList {
	let mut list: TokenList = TokenList::new(input_path, input_source);
	let source : &str = list.context.source();
	let source_path : &std::path::Path = list.context.path();
	let spanned_tokens = TokenName::lexer(list.context.source()).spanned();
	for tok_and_range in spanned_tokens {
		match tok_and_range {
			(lex::TokenName::Error, range) => {
				panic!("{:?}:({:?},{:?}) - Error B00000\n\tunrecognized sequence of text during scanning/lexing of sequence '{}'", &source_path, list.context.human_line(), list.context.human_column(), &source[range.start..range.end])
			},
			(token_name , range) => {
				let progress: usize = range.end - range.start;
				list.context.advance_cursor(progress);
				let start = list.context.mark();
				let span_marker = list.context.span(start);
				let token: Token = lex::Token{name: token_name, span: span_marker};
				list.tokens.push(token);
			},
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
