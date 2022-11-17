use logos::Logos;
use std::path::PathBuf;

use crate::context;
use crate::state;

#[derive(Debug)]
pub enum CommentStyle {
	Line,
	Block,
}

#[derive(Debug)]
pub enum LineWhitespaceStyle {
	CarriageReturn = 0x01,
	LineFeed = 0x02,
	CarriageReturnLineFeed = 0x3,
}

#[derive(Logos, Debug)]
pub enum TokenName {
	#[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
	Identifier,
	#[regex(r"[0-9]+")]
	Number,
	#[regex(r"'[^'\\]*(?:\\.[^'\\]*)*'")]
	StringLiteral,
	// Keywords
	#[token("if")]
	If,
	#[token("auto")]
	Auto,
	#[token("extrn")]
	Extrn,
	#[token("case")]
	Case,
	#[token("while")]
	While,
	#[token("switch")]
	Switch,
	#[token("goto")]
	Goto,
	#[token("return")]
	Return,
	// Punctuator
	#[token("(")]
	LeftParen,
	#[token(")")]
	RightParen,
	#[token("{")]
	LeftBrace,
	#[token("}")]
	RightBrace,
	#[token(";")]
	Semicolon,
	#[token(",")]
	Comma,
	#[token("'")]
	SingleQuote,
	#[token("\"")]
	Quote,
	#[token("+")]
	Plus,
	#[token("++")]
	PlusPlus,
	#[token("-")]
	Minus,
	#[token("--")]
	MinusMinus,
	#[token("!")]
	Exclamation,
	#[token("&")]
	Ampersand,
	#[token("|")]
	VerticalBar,
	#[token("=")]
	Equals,
	#[token("==")]
	EqualsEquals,
	#[token("!=")]
	ExclamationEquals,
	#[token("<")]
	LessThan,
	#[token("<<")]
	LessThanLessThan,
	#[token("<=")]
	LessThanEquals,
	#[token(">")]
	GreaterThan,
	#[token(">>")]
	GreaterThanGreaterThan,
	#[token(">=")]
	GreaterThanEquals,
	#[token("%")]
	Percent,
	#[token("*")]
	Asterisks,
	#[token("/")]
	ForwardSlash,
	#[regex(r"//[^\r\n]*", |_lex| Some(CommentStyle::Line))]
	#[regex(r"/\*([^*]|\**[^*/])*\*+/", |_lex| Some(CommentStyle::Block))]
	Comment(CommentStyle),
	#[regex(r"[ \t\f]+")]
	Whitespace,
	#[regex(r"(\r)?(\n)?", |lex| match lex.slice() {
		"\r\n" => Some(LineWhitespaceStyle::CarriageReturnLineFeed),
		"\r" => Some(LineWhitespaceStyle::CarriageReturn),
		"\n" => Some(LineWhitespaceStyle::LineFeed),
		_ => None
	})]
	LineWhitespace(LineWhitespaceStyle),
	#[regex(r"/\*([^*]|\*+[^*/])*\*?")]
	#[error]
	Error,
}

#[derive(Debug)]
pub struct Token {
	pub name: TokenName,
	pub span: context::Span,
}

#[derive(Debug)]
pub enum TokenCategory {
	Identifier,
	NumericLiteral,
	StringLiteral,
	Keyword,
	Punctuator,
	Whitespace,
	LineWhitespace,
	Error,
}

impl Token {
	pub fn categorize(&self) -> TokenCategory {
		match self.name {
			TokenName::Identifier => TokenCategory::Identifier,
			TokenName::Number => TokenCategory::NumericLiteral,
			TokenName::StringLiteral => TokenCategory::StringLiteral,
			// Keywords
			TokenName::If
			| TokenName::Auto
			| TokenName::Extrn
			| TokenName::Case
			| TokenName::While
			| TokenName::Switch
			| TokenName::Goto
			| TokenName::Return => TokenCategory::Keyword,
			// Punctuator
			TokenName::LeftParen
			| TokenName::RightParen
			| TokenName::LeftBrace
			| TokenName::RightBrace
			| TokenName::Semicolon
			| TokenName::Comma
			| TokenName::SingleQuote
			| TokenName::Quote
			| TokenName::Plus
			| TokenName::PlusPlus
			| TokenName::Minus
			| TokenName::MinusMinus
			| TokenName::Exclamation
			| TokenName::Ampersand
			| TokenName::VerticalBar
			| TokenName::Equals
			| TokenName::EqualsEquals
			| TokenName::ExclamationEquals
			| TokenName::LessThan
			| TokenName::LessThanLessThan
			| TokenName::LessThanEquals
			| TokenName::GreaterThan
			| TokenName::GreaterThanGreaterThan
			| TokenName::GreaterThanEquals
			| TokenName::Percent
			| TokenName::Asterisks
			| TokenName::ForwardSlash => TokenCategory::Punctuator,
			// Whitespace
			TokenName::Whitespace | TokenName::Comment(_) => {
				TokenCategory::Whitespace
			}
			TokenName::LineWhitespace(_) => TokenCategory::LineWhitespace,
			TokenName::Error => TokenCategory::Error,
		}
	}
}

#[derive(Debug)]
pub struct TokenList {
	pub tokens: Vec<Token>,
	pub context: context::Context,
}

impl TokenList {
	pub fn new(path: PathBuf, source: String) -> TokenList {
		Self {
			tokens: Vec::new(),
			context: context::Context::new(path, source),
		}
	}
}

/// All lexer errors start with B1, and then are followed by any number of hex
/// characters. There is no bit pattern among them that means anything.
#[derive(Debug, Clone, Copy)]
pub enum Error {
	UnrecognizedToken = 0x0000,
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let integer_value = *self as u32;
		f.write_fmt(format_args!("B1-{:04x} - ", integer_value))?;
		match self {
			Error::UnrecognizedToken => f.write_str("Unrecognized token"),
		}
	}
}

impl std::fmt::Display for TokenName {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			TokenName::Identifier => "Identifier",
			TokenName::Number => "Number",
			TokenName::StringLiteral => "StringLiteral",
			// Keywords
			TokenName::If => "if",
			TokenName::Auto => "auto",
			TokenName::Extrn => "extrn",
			TokenName::Case => "case",
			TokenName::While => "while",
			TokenName::Switch => "switch",
			TokenName::Goto => "goto",
			TokenName::Return => "return",
			// Punctuator
			TokenName::LeftParen => "Left Parenthesis",
			TokenName::RightParen => "Right Parenthesis",
			TokenName::LeftBrace => "Left Brace",
			TokenName::RightBrace => "Right Brace",
			TokenName::Semicolon => "Semicolon",
			TokenName::Comma => "Comma",
			TokenName::SingleQuote => "Single Quotation Mark",
			TokenName::Quote => "Double Quotation Mark",
			TokenName::Plus => "Plus",
			TokenName::PlusPlus => "Plus Plus",
			TokenName::Minus => "Minus",
			TokenName::MinusMinus => "Minus Minus",
			TokenName::Exclamation => "Exclamation",
			TokenName::Ampersand => "Ampersand",
			TokenName::VerticalBar => "Vertical Bar",
			TokenName::Equals => "Equals",
			TokenName::EqualsEquals => "Equals Equals",
			TokenName::ExclamationEquals => "Exclamation Equals",
			TokenName::LessThan => "Less Than",
			TokenName::LessThanLessThan => "Less Less Than",
			TokenName::LessThanEquals => "Less Than or Equals",
			TokenName::GreaterThan => "Greater Than",
			TokenName::GreaterThanGreaterThan => "Greater Greater Than",
			TokenName::GreaterThanEquals => "Greater than or Equals",
			TokenName::Percent => "Percent",
			TokenName::Asterisks => "Asterisks",
			TokenName::ForwardSlash => "Forward Slash",
			// Whitespace
			TokenName::Whitespace => "Whitespace",
			TokenName::Comment(style) => match style {
				CommentStyle::Line => "Line Comment",
				CommentStyle::Block => "Block Comment",
			},
			TokenName::LineWhitespace(style) => match style {
				LineWhitespaceStyle::CarriageReturn => " Newline \\r",
				LineWhitespaceStyle::LineFeed => "Newline \\n",
				LineWhitespaceStyle::CarriageReturnLineFeed => {
					"Newline \\r\\n"
				}
			},
			TokenName::Error => "Unrecognized Token",
		})
	}
}

/// Given the following source, parse it into a list of tokens.
///
/// This function will use a context to perform its allocations
/// where necessary, except in the case of warnings or errors which
/// may perform some allocations on the path to (possibly printing) errors.
///
/// if config.print_tokens is set, this function will also print tokens to
/// the designated `config.print_tokens_output` location.
pub fn lex(
	input_path: PathBuf,
	input_source: String,
	config: &state::CompilationConfiguration,
) -> TokenList {
	let mut list: TokenList = TokenList::new(input_path, input_source);
	let source: &str = list.context.source();
	let source_path: &std::path::Path = list.context.path();
	let spanned_tokens = TokenName::lexer(list.context.source()).spanned();
	for (token_name, range) in spanned_tokens {
		if let TokenName::Error = token_name {
			eprintln!(
				"{:?}[{:?},{:?}] - Error {}\n\tunrecognized input text during scanning/lexing of sequence '{}'",
				&source_path, list.context.human_line(),
				list.context.human_column(),
				Error::UnrecognizedToken,
				&source[range.start..range.end]);
			panic!()
		}
		let progress: usize = range.end - range.start;
		let token_span = list.context.next_span(progress);
		let token: Token = Token {
			name: token_name,
			span: token_span,
		};
		list.tokens.push(token);
	}
	if config.print_tokens {
		for token in &list.tokens {
			match token.categorize() {
				TokenCategory::LineWhitespace | TokenCategory::Whitespace => {
					print!(
						"[{} {}]",
						token.name,
						token.span.display_range(&list.context)
					)
				}
				TokenCategory::Keyword => {
					print!(
						"[Keyword {} {}]",
						token.name,
						token.span.display_range(&list.context)
					)
				}
				_ => {
					print!(
						"[{} {} {}]",
						token.name,
						token.span.display_range(&list.context),
						token.span.text(&list.context)
					)
				}
			}
			if list
				.tokens
				.last()
				.is_some_and(|last_token| !std::ptr::eq(&token, &last_token))
			{
				print!(" ");
			}
			if let TokenName::LineWhitespace(_) = token.name {
				println!();
			}
		}
	}
	list
}
