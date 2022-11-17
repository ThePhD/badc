//! AST types and parsing contexts, for keeping track of source code spans.
//!
//! The AST nodes somewhat reflect the canonical syntax specified in
//! <https://www.bell-labs.com/usr/dmr/www/kbman.pdf> S2.1, with extensions.

use clap::ValueEnum;
use std::fs::File;
use std::io::{Stdin, Stdout};
use std::path::PathBuf;

#[derive(ValueEnum, Debug, Clone)]
pub enum VerbosityLevel {
	Silent = 0,
	Trace = 1,
	Debug = 2,
}

/// Defines various verbosity levels for individual stages of the compiler.
#[derive(Debug, Clone)]
pub struct VerbosityLevels {
	/// The level of verbosity when turning input into a token list.
	pub lex_verbosity_level: VerbosityLevel,
	/// The level of verbosity when selecting, combining, and massaging tokens
	/// into a synax tree.
	pub parse_verbosity_level: VerbosityLevel,
	/// The level of verbosity for actions executed directly on the parse tree
	/// (such as simplifications or compile-time execution).
	pub parse_execute_verbosity_level: VerbosityLevel,
	/// The level of verbosity when turning a ;arse tree into a specific output
	/// (code generation, translation to another language, or otherwise).
	pub generate_verbosity_level: VerbosityLevel,
}

#[derive(Debug)]
pub enum ProgramSource {
	Path(PathBuf),
	File(File),
	Stdin(Stdin),
}

#[derive(Debug)]
pub enum ProgramSink {
	Path(PathBuf),
	File(File),
	Stdout(Stdout),
}

/// This is a very long documentation comment that is meant to be forced down to
/// 80 columns.
#[derive(Debug)]
pub struct CompilationConfiguration {
	pub input: ProgramSource,
	pub verbosity_level: VerbosityLevel,
	pub verbosity_levels: VerbosityLevels,
	pub print_tokens: bool,
	pub print_ast: bool,
	pub output: ProgramSink,
	pub print_tokens_output: ProgramSink,
	pub print_ast_output: ProgramSink,
}

impl std::fmt::Display for VerbosityLevel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			VerbosityLevel::Silent => "silent",
			VerbosityLevel::Trace => "trace",
			VerbosityLevel::Debug => "debug",
		})
	}
}
