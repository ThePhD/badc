//! AST types and parsing contexts, for keeping track of source code spans.
//!
//! The AST nodes somewhat reflect the canonical syntax specified in
//! <https://www.bell-labs.com/usr/dmr/www/kbman.pdf> S2.1, with extensions.

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

	// All AST nodes are allocated on this arena, to avoid hammering the heap and
	// so that all nodes can simply contain references and slices directly, to
	// aid pattern-matching.
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

		let index: u32 = spans
			.raw_spans
			.len()
			.try_into()
			.expect("ran out of span indices");
		Span(index)
	}

	/// Advances the cursor.
	///
	/// This function takes `&self` because as AST nodes are created, they will
	/// hold references into the arena, which locks up a lifetime for the overall
	/// context, disallowing any `&mut` operations.
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
}

/// A B program.
///
/// Corresponds to `program` in the B grammar.
pub struct Program<'ctx> {
	/// Definitions in this program.
	///
	/// Note that there are no declarations; unlike in C, B declares names via
	/// `extrn` statements in function bodies.
	pub defs: &'ctx [Def<'ctx>],
}

/// A global variable or a function.
///
/// Corresponds to `definition` in the B grammar.
pub enum Def<'ctx> {
	/// A global variable.
	Global(Global<'ctx>),
	/// A function definition.
	Func(Func<'ctx>),
}

/// A global variable.
///
/// Corresponds to part of `definition` in the B grammar.
pub struct Global<'ctx> {
	/// The name of the variable.
	pub name: Id<'ctx>,
	/// The declared size of this variable, if it is an array.
	pub size: Option<(ArraySize<'ctx>, Span)>,
	/// The initializers for this variable.
	pub inits: &'ctx [InitVal<'ctx>],
	/// The overall span.
	pub span: Span,
}

/// The declared size of an array.
pub enum ArraySize<'ctx> {
	/// The syntax `name[]`, which declares an array of the same size as the
	/// initializer that follows.
	Implicit,
	/// The syntax `name[n]`, which declares an array of a fixed size, possibly
	/// larger than the initializer that follows.
	Explicit(Const<'ctx>),
}

/// An initializer: an "atomic" expression.
///
/// Corresponds to `ival` in the B grammar.
pub enum InitVal<'ctx> {
	/// A reference to another symbol.
	Id(Id<'ctx>),
	/// A constant value.
	Const(Const<'ctx>),
}

/// A function definition.
///
/// Corresponds to part of `definition` in the B grammar.
pub struct Func<'ctx> {
	/// The name of the function.
	pub name: Id<'ctx>,
	/// The function's named parameters.
	pub params: &'ctx [Id<'ctx>],
	/// The statements that make up the body.
	///
	/// Note that this is *not* a block statement!
	pub body: &'ctx [Stmt<'ctx>],
	/// The overall span.
	pub span: Span,
}

/// A statement.
///
/// Corresponds to `statement` in the B grammar.
///
/// We represent statements in a slightly different way from the grammar. In the
/// grammar, all statements that are not blocks, goto, return, or an
/// expression are followed by another statement, so it makes more sense to
/// refactor this so that blocks and function bodies are lists of statements.
///
/// This makes some strictly non-conforming syntax trees expressible, but we're
/// likely going to support them as extensions anyways.
pub struct Stmt<'ctx> {
	/// The kind of expression this is.
	pub kind: StmtKind<'ctx>,
	/// The overall span.
	pub span: Span,
}

/// A type of statement.
pub enum StmtKind<'ctx> {
	/// A variable declaration: e.g. `auto x, y, z;`.
	Auto {
		/// The declarations; each one may have an associated constant, which acts
		/// as the initializer. E.g., `auto x 42;`
		decls: &'ctx [(Id<'ctx>, Option<Const<'ctx>>)],
	},
	/// An external symbol declaration: e.g. `extrn getvec;`
	Extrn {
		/// The list of symbols declared.
		decls: &'ctx [Id<'ctx>],
	},
	/// A label: e.g. `foo:`.
	Label(Id<'ctx>),
	/// A case label: e.g. `case 42:`.
	Case(Const<'ctx>),
	/// A block of statements: e.g. `{ foo; bar; baz; }`.
	Block(&'ctx [Stmt<'ctx>]),
	/// An if statement: e.g. `if (maybe()) return (42);`
	If {
		/// The statement's condition.
		cond: Expr<'ctx>,
		/// The body executed if `cond` is true.
		body: &'ctx Stmt<'ctx>,
		/// The optional body executed if `cond` is false.
		elze: Option<&'ctx Stmt<'ctx>>,
	},
	/// A while loop, e.g. `while (1 == 1) foo();`.
	While {
		/// The statement's condition.
		cond: Expr<'ctx>,
		/// The loop body.
		body: &'ctx Stmt<'ctx>,
	},
	/// A switch statement: e.g. `switch (1) case 1: 5;`.
	Switch {
		/// The value being switched on.
		switchee: Expr<'ctx>,
		/// The body of the switch, which, surprisingly, is permitted to be any
		/// expression.
		body: &'ctx Stmt<'ctx>,
	},
	/// A goto statement: e.g. `goto somewhere;`.
	Goto(Expr<'ctx>),
	/// A return statement: e.g. `return (something);`.
	Return(Option<Expr<'ctx>>),
	/// A plain expression at statement scope: e.g. `foo;`.
	Expr(Expr<'ctx>),
	/// An empty statement: `;`.
	Empty,
}

/// An expression.
///
/// Corresponds to `rvalue` and `lvalue` in the B grammar.
///
/// The lvalue/rvalue distinction is not deeply useful in a parsing context so
/// they are merged into one here.
pub struct Expr<'ctx> {
	/// The kind of expression this is.
	pub kind: ExprKind<'ctx>,
	/// The overall span.
	pub span: Span,
}

/// A type of expression.
pub enum ExprKind<'ctx> {
	/// A parenthesized expresion, e.g. `(x + y)`.
	Parens(&'ctx Expr<'ctx>),
	/// A simple expression.
	InitVal(InitVal<'ctx>),
	/// A dereference operation, e.g. `*p`.
	Deref {
		/// The "pointer" value being dereferenced.
		ptr: &'ctx Expr<'ctx>,
	},
	/// An indexing operation, e.g. `p[55]`.
	Index {
		/// The "pointer" value being offset and dereferenced.
		ptr: &'ctx Expr<'ctx>,
		/// The indexing value.
		index: &'ctx Expr<'ctx>,
	},
	/// A plain assignment, e.g. `x = y`
	Assign {
		/// The left-hand side.
		lhs: &'ctx Expr<'ctx>,
		/// The right-hand side.
		rhs: &'ctx Expr<'ctx>,
	},
	/// An unary expression.
	///
	/// This includes increment and decrement expressions.
	Unary {
		/// The expression being operated on.
		expr: &'ctx Expr<'ctx>,
		/// The operation.
		kind: UnaryOp,
	},
	/// A binary operation.
	///
	/// This includes assignment expressions.
	Binary {
		/// The left-hand side.
		lhs: &'ctx Expr<'ctx>,
		/// The right-hand side.
		rhs: &'ctx Expr<'ctx>,
		/// The operation.
		kind: BinaryOp,
		/// If set, this is an assignment operation, such as `x =<= y;`
		is_assign: bool,
	},
	/// A ternary (i.e., a select).
	Ternary {
		/// The condition to select on.
		cond: &'ctx Expr<'ctx>,
		/// The value to produce on success.
		yes: &'ctx Expr<'ctx>,
		/// The value to produce on failure.
		no: &'ctx Expr<'ctx>,
	},
	/// A function call.
	Call {
		/// The function to call.
		func: &'ctx Expr<'ctx>,
		/// The arguments to pass into the call.
		args: &'ctx [&'ctx Expr<'ctx>],
	},
}

/// An unary operation.
pub enum UnaryOp {
	/// `++x`.
	PreInc,
	/// `--x`.
	PreDec,
	/// `x++`.
	PostInc,
	/// `x--`.
	PostDec,
	/// `-x`.
	Minus,
	/// `!x`.
	Not,
}

pub enum BinaryOp {
	/// `x | y`.
	Or,
	/// `x & y`.
	And,
	/// `x == y`.
	Eq,
	/// `x != y`.
	Ne,
	/// `x > y`.
	Gt,
	/// `x >= y`.
	Ge,
	/// `x < y`.
	Lt,
	/// `x <= y`.
	Le,
	/// `x << y`.
	Shl,
	/// `x >> y`.
	Shr,
	/// `x + y`.
	Add,
	/// `x - y`.
	Sub,
	/// `x % y`.
	Rem,
	/// `x * y`.
	Mul,
	/// `x / y`.
	Div,
}

/// A named identifier.
///
/// Corresponds to `name` in the B grammar.
pub struct Id<'ctx> {
	/// The name of the identifier.
	pub name: &'ctx str,
	/// The identifier's span.
	pub span: Span,
}

/// An (unsigned!) integer constant.
pub struct Int {
	/// The value of the constant.
	pub value: u128,
	/// The constant's span.
	pub span: Span,
}

/// A character constant.
pub struct Char {
	/// The value of the constant.
	pub value: u8,
	/// The constant's span.
	pub span: Span,
}

/// A string constant.
pub struct Str<'ctx> {
	/// The value of the constant.
	pub value: &'ctx str,
	/// The constant's span.
	pub span: Span,
}

/// A constant of some kind.
///
/// Corresponds to `constant` in the B grammar.
pub enum Const<'ctx> {
	/// An integer constant.
	Int(Int),
	/// A character constant.
	Char(Char),
	/// A string constant.
	Str(Str<'ctx>),
}
