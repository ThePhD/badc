pub mod bad {
	pub enum ProgramSource {
		Path(std::ffi::OsString),
		File(std::fs::File),
		Stdin(std::io::Stdin),
	}
	
	pub struct CompilationConfiguration {
		pub sources: std::vec::Vec<ProgramSource>
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
		Identifier(std::string::String)

	}

	#[derive(Debug)]
	pub struct TokenList {
		pub tokens : std::vec::Vec<Token>
	}

	#[derive(Debug)]
	pub struct SyntaxTree {

	}

	pub fn lex (_config: &CompilationConfiguration) -> TokenList {
		let list: TokenList = TokenList { tokens: std::vec::Vec::new() };
		list
	}

	pub fn parse (token_stream: TokenList, _config: &CompilationConfiguration) -> SyntaxTree {
		let tree = SyntaxTree {  };
		tree
	}
	
	pub fn compile (config: &CompilationConfiguration) -> SyntaxTree {
		let lex: TokenList = lex(&config);
		let tree: SyntaxTree = parse(lex, &config);
		tree
	}	
}
