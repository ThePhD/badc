use std::path::PathBuf;

use bad::CompilationConfiguration;
use bad::ProgramSource;

pub struct CompilationOptions {
	pub sources: Vec<ProgramSource>,
}

fn main() {
	// TODO: parse from Command Line and/or JSON
	let options = CompilationOptions {
		sources: vec![ProgramSource::Path(PathBuf::from("./main.b"))],
	};
	// TODO: properly transfer compilation options to library configuration
	let config = CompilationConfiguration {
		sources: options.sources,
	};
	let tree = bad::compile(&config);
	println!("{:?} 🎉!", tree);
}
