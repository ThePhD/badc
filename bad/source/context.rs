use std::cell::RefCell;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;

use bumpalo::Bump;

/// A source code span.
///
/// Internally this is just an ID; in order to obtain information about the
/// span, it must be queried from a corresponding [`Context`].
#[derive(Copy, Clone, Debug)]
pub struct Span(u32);

impl Span {
	/// Returns the byte range for this span.
	pub fn range(self, ctx: &Context) -> (usize, usize) {
		ctx.spans.borrow().raw_spans[self.0 as usize].range
	}

	/// Returns the textual contents of this span as a string slice.
	pub fn text(self, ctx: &Context) -> &str {
		let (start, end) = self.range(ctx);
		&ctx.source[start..end]
	}

	/// Returns the coordinates (line and column) that this span starts at.
	///
	/// Line and column are zero-indexed; you may want to one-index them for
	/// pretty-printing.
	pub fn coords(self, ctx: &Context) -> (u32, u32) {
		let raw = &ctx.spans.borrow().raw_spans[self.0 as usize];
		(raw.line, raw.col)
	}

	/// See [`Span::coords()`].
	pub fn line_number(self, ctx: &Context) -> u32 {
		self.coords(ctx).0
	}

	/// See [`Span::coords()`].
	pub fn col_number(self, ctx: &Context) -> u32 {
		self.coords(ctx).1
	}

	/// Uses the given `Context` to produce a [`fmt::Display`]able value.
	///
	/// `Span` itself cannot be [`fmt::Display`], because we need a matching
	/// `Context` to interpret it with.
	pub fn display(self, ctx: &Context) -> impl fmt::Display + '_ {
		struct Displayable<'ctx> {
			span: Span,
			ctx: &'ctx Context,
		}
		impl fmt::Display for Displayable<'_> {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				let (line, col) = self.span.coords(self.ctx);
				write!(
					f,
					"{}[{}:{}]",
					self.ctx.path().display(),
					line + 1,
					col + 1
				)
			}
		}
		Displayable { span: self, ctx }
	}

	/// Uses the given `Context` to produce a [`fmt::Display`]able value,
	/// particular for a range.
	///
	/// `Span` itself cannot be [`fmt::Display`], because we need a matching
	/// `Context` to interpret it with.
	pub fn display_range(self, ctx: &Context) -> impl fmt::Display + '_ {
		struct Displayable<'ctx> {
			span: Span,
			ctx: &'ctx Context,
		}
		impl fmt::Display for Displayable<'_> {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				let (line, col) = self.span.coords(self.ctx);
				write!(f, "[{}:{}]", line + 1, col + 1)
			}
		}
		Displayable { span: self, ctx }
	}
}

/// A position in the source code marking the start of a `Span`.
#[derive(Copy, Clone, Default, Debug)]
pub(crate) struct Mark {
	offset: usize,
	line: u32,
	col: u32,
}

/// Internal representation of information associated with a span.
///
/// Currently stored as AoS, but SoA may be a viable future optimization.
#[derive(Debug)]
struct RawSpan {
	range: (usize, usize),
	line: u32,
	col: u32,
}

/// State for generating spans. This is broken out into a separate struct so
/// that we can wrap it in a RefCell.
#[derive(Default, Debug)]
struct SpanState {
	// `Span`s index into this array.
	raw_spans: Vec<RawSpan>,
	// The cursor for tracking marks and creating spans.
	cursor: Mark,
}

/// A parsing context.
///
/// Keeps track of memory allocation, source code spans, and miscellaneous
/// book-keeping for an AST.
#[derive(Debug)]
pub struct Context {
	path: PathBuf,
	source: String,

	// All AST nodes are allocated on this arena, to avoid hammering the heap
	// and so that all nodes can simply contain references and slices directly,
	// to aid pattern-matching.
	pub(crate) arena: Bump,
	spans: RefCell<SpanState>,
}

impl Context {
	/// Creates a new parsing context over the given path and source.
	pub fn new(path: PathBuf, source: String) -> Context {
		Self {
			path,
			source,
			arena: Bump::new(),
			spans: Default::default(),
		}
	}

	/// Returns the path to the source file.
	pub fn path(&self) -> &Path {
		&self.path
	}

	/// Returns the path to the contents of the source file.
	pub fn source(&self) -> &str {
		&self.source
	}

	pub fn offset(&self) -> usize {
		self.spans.borrow().cursor.offset
	}

	pub fn column(&self) -> u32 {
		self.spans.borrow().cursor.col
	}

	pub fn line(&self) -> u32 {
		self.spans.borrow().cursor.line
	}

	pub fn human_column(&self) -> u32 {
		self.column() + 1
	}

	pub fn human_line(&self) -> u32 {
		self.line() + 1
	}

	pub(crate) fn unread(&self) -> &str {
		&self.source[self.spans.borrow().cursor.offset..]
	}

	/// Creates a new mark pointing to the current position in the source.
	pub(crate) fn mark(&self) -> Mark {
		self.spans.borrow().cursor
	}

	/// Creates a new span using the given mark as the starting point.
	pub(crate) fn span(&self, start: Mark) -> Span {
		let mut spans = self.spans.borrow_mut();
		let end = spans.cursor.offset;
		spans.raw_spans.push(RawSpan {
			range: (start.offset, end),
			line: start.line,
			col: start.col,
		});

		let index: u32 = (spans.raw_spans.len() - 1)
			.try_into()
			.expect("ran out of span indices");
		Span(index)
	}

	/// Advances the cursor.
	///
	/// This function takes `&self` because as AST nodes are created, they will
	/// hold references into the arena, which locks up a lifetime for the
	/// overall context, disallowing any `&mut` operations.
	///
	/// # Panics
	///
	/// Panics if `len > self.unread().len()`.
	pub(crate) fn advance_cursor(&self, len: usize) {
		let mut spans = self.spans.borrow_mut();
		let offset = spans.cursor.offset;
		for c in self.source[offset..offset + len].chars() {
			if c == '\n' {
				spans.cursor.line += 1;
				spans.cursor.col = 0;
			} else {
				spans.cursor.col += 1;
			}
		}
		spans.cursor.offset += len;
	}

	/// Creates a new mark, advances the cursor, and returns a span representing
	/// the new marking plus cursor advancement.
	///
	/// It is effectively a shortcut for calling mark, advance_cursor, and span
	/// in-order.
	///
	/// # Panics
	///
	/// Panics if `len > self.unread().len()`.
	pub(crate) fn next_span(&self, len: usize) -> Span {
		let start = self.mark();
		self.advance_cursor(len);
		self.span(start)
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
	MissingFile = 0x0000,
	UnreadableFile = 0x0001,
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("B0-{:04x} - ", *self as u32))?;
		match self {
			Error::MissingFile => f.write_str("Missing file"),
			Error::UnreadableFile => f.write_str("Unable to read file"),
		}
	}
}
