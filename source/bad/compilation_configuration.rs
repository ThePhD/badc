pub mod bad {
	use std::vec;
	use std::ffi;

	pub enum program_source {
		file_path(std::ffi::OsString),
		file_stream,
		stdin,
	}

	pub struct compilation_configuration {
		sources: std::Vec<program_source>
	}
}
