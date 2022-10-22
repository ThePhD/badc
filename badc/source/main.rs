use std::{path::PathBuf, str::FromStr};
use clap::Parser;

/// The badc compiler for the B language (Kernighan, 1969). Learning experiment for DrawsMiguel and ThePhD on Rust and some compilation techniques. Released un the CC0 1.0 Universal (e.g. Public Domain dedication).
#[derive(Parser, Debug)]
#[command(author, version, about = "A B language (Kernighan, 1969) compiler. Not at all useful.")]
struct CommandLineCompilationOptions {
	/// All of the paths to the input to compile, each one considered an independent translation unit.
	#[arg(short, long)]
	inputs: Vec<PathBuf>,

	/// Print out the token sequence.
	#[arg(short, long, default_value_t = true)]
	print_tokens: bool,

	/// Print out the AST.
	#[arg(short, long, default_value_t = true)]
	print_ast: bool,

	/// The path to the output.
	#[arg(short, long)]
	output: Option<PathBuf>,
}

fn main() {
	let mut args = CommandLineCompilationOptions::parse();
	if args.inputs.is_empty() {
		args.inputs.push(PathBuf::from_str("./main.b").unwrap());
	}
	for input in args.inputs {
		let output = match args.output {
			Some(ref target_path) => target_path.clone(),
			None => { let mut target_path = input.clone(); target_path.push(".out"); target_path },
		};	
		let config = bad::CompilationConfiguration{
			input: bad::ProgramSource::Path(input.clone()),
			print_tokens: args.print_tokens,
			print_ast: args.print_ast,
			output: bad::ProgramSink::Path(output),
		};
		let tree = bad::compile(&config);
		println!("{:?} 🎉!", tree);
	}
}
