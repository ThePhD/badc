pub mod badc {
	pub struct CompilationOptions {
		pub sources: std::vec::Vec<bad::bad::ProgramSource>,
	}
}

fn main () -> () {
	let options: badc::CompilationOptions = badc::CompilationOptions{ sources :vec!(bad::bad::ProgramSource::Stdin(std::io::stdin()))};
	let config : bad::bad::CompilationConfiguration = bad::bad::CompilationConfiguration{ sources: options.sources };
	println!("{} 🎉!", bad::bad::compile(config));
}
