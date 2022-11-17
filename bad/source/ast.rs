//! AST types and parsing contexts, for keeping track of source code spans.
//!
//! The AST nodes somewhat reflect the canonical syntax specified in
//! <https://www.bell-labs.com/usr/dmr/www/kbman.pdf> S2.1, with extensions.

use crate::context::Span;

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
