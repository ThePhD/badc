use logos::Logos;

use crate::context;

#[derive(Logos, Debug)]
pub enum TokenName {
	#[regex(r"[ \t\f]+")]
	Whitespace,
	#[regex(r"(\r)?(\n)?")]
	LineWhitespace,
	#[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
	Identifier,
	#[regex(r"[0-9]+")]
	Number,
	#[regex(r"'[^'\\]*(?:\\.[^'\\]*)*'")]
	StringLiteral,
	#[token("/*")]
	CommentStart,
	#[token("*/")]
	CommentEnd,
	//#[regex(r"/\*.*\*/")]
	//Comment,
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
	// Symbols
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
	#[error]
	Error,
}

#[derive(Debug)]
pub struct Token {
	pub name: TokenName,
	pub span: context::Span
}
