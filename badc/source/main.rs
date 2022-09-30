pub mod badc {
	pub struct CompilationOptions {
		pub sources: std::vec::Vec<bad::bad::ProgramSource>,
	}
}

fn main () -> () {
	// TODO: parse from Command Line and/or JSON
	let options: badc::CompilationOptions = badc::CompilationOptions{ sources :vec!(bad::bad::ProgramSource::Path(std::ffi::OsString::from("./main.b"))) };
	// TODO: properly transfer compilation options to library configuration
	let config : bad::bad::CompilationConfiguration = bad::bad::CompilationConfiguration{ sources: options.sources };
	let tree : bad::bad::SyntaxTree = bad::bad::compile(&config);
	println!("{:?} 🎉!", tree);
}
