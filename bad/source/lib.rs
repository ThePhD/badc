#![feature(is_some_and)]

use std::io::*;
use std::path::PathBuf;

pub mod ast;
pub mod context;
pub mod lex;
pub mod state;

#[derive(Debug)]
pub struct SyntaxTree {}

pub fn get_source_text(source: &mut state::ProgramSource) -> (PathBuf, String) {
	match source {
		state::ProgramSource::Path(pathbuf) => (
			pathbuf.clone(),
			std::fs::read_to_string(pathbuf.as_path()).unwrap(),
		),
		state::ProgramSource::File(file) => {
			let mut buf: String = String::new();
			file.read_to_string(&mut buf).unwrap();
			("<file?handle?>".into(), buf)
		}
		state::ProgramSource::Stdin(stdin) => {
			let mut buf: String = String::new();
			stdin.read_to_string(&mut buf).unwrap();
			("<stdin>".into(), buf)
		}
	}
}

pub fn parse(
	_token_stream: lex::TokenList,
	_config: &state::CompilationConfiguration,
) -> SyntaxTree {
	SyntaxTree {}
}

pub fn compile(mut config: state::CompilationConfiguration) -> SyntaxTree {
	let source = &mut config.input;
	let (source_path, source_text) = get_source_text(source);
	let lex: lex::TokenList = lex::lex(source_path, source_text, &config);
	let tree: SyntaxTree = parse(lex, &config);
	tree
}
