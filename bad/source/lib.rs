pub mod bad {
	pub enum ProgramSource {
		Path(std::ffi::OsString),
		File(std::fs::File),
		Stdin(std::io::Stdin),
	}
	
	pub struct CompilationConfiguration {
		pub sources: std::vec::Vec<ProgramSource>
	}
	
	pub fn compile (_config: CompilationConfiguration) -> i64 {
		500
	}	
}
