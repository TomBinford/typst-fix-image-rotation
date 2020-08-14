//! Mapping of values to the locations they originate from in source code.

use std::fmt::{self, Debug, Formatter};
use std::ops::{Add, Sub};

#[cfg(feature = "serialize")]
use serde::Serialize;

/// Span offsetting.
pub trait Offset {
    /// Offset all spans contained in `Self` by the given position.
    fn offset(self, by: Pos) -> Self;
}

/// A vector of spanned values of type `T`.
pub type SpanVec<T> = Vec<Spanned<T>>;

impl<T> Offset for SpanVec<T> {
    fn offset(mut self, by: Pos) -> Self {
        for spanned in &mut self {
            spanned.span = spanned.span.offset(by);
        }
        self
    }
}

/// A value with the span it corresponds to in the source code.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Spanned<T> {
    /// The value.
    pub v: T,
    /// The corresponding span.
    pub span: Span,
}

impl<T> Spanned<T> {
    /// Create a new instance from a value and its span.
    pub fn new(v: T, span: Span) -> Self {
        Self { v, span }
    }

    /// Create a new instance from a value with the zero span.
    pub fn zero(v: T) -> Self {
        Self { v, span: Span::ZERO }
    }

    /// Access the value.
    pub fn value(self) -> T {
        self.v
    }

    /// Map the value using a function while keeping the span.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned { v: f(self.v), span: self.span }
    }

    /// Maps the span while keeping the value.
    pub fn map_span(mut self, f: impl FnOnce(Span) -> Span) -> Self {
        self.span = f(self.span);
        self
    }
}

impl<T> Offset for Spanned<T> {
    fn offset(self, by: Pos) -> Self {
        self.map_span(|span| span.offset(by))
    }
}

impl<T: Debug> Debug for Spanned<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.v.fmt(f)?;
        if f.alternate() {
            f.write_str(" ")?;
            self.span.fmt(f)?;
        }
        Ok(())
    }
}

/// Locates a slice of source code.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Span {
    /// The inclusive start position.
    pub start: Pos,
    /// The inclusive end position.
    pub end: Pos,
}

impl Span {
    /// The zero span.
    pub const ZERO: Self = Self { start: Pos::ZERO, end: Pos::ZERO };

    /// Create a new span from start and end positions.
    pub fn new(start: Pos, end: Pos) -> Self {
        Self { start, end }
    }

    /// Create a span including just a single position.
    pub fn at(pos: Pos) -> Self {
        Self { start: pos, end: pos }
    }

    /// Create a new span with the earlier start and later end position.
    pub fn merge(a: Self, b: Self) -> Self {
        Self {
            start: a.start.min(b.start),
            end: a.end.max(b.end),
        }
    }

    /// Expand a span by merging it with another span.
    pub fn expand(&mut self, other: Self) {
        *self = Self::merge(*self, other)
    }
}

impl Offset for Span {
    fn offset(self, by: Pos) -> Self {
        Self {
            start: self.start.offset(by),
            end: self.end.offset(by),
        }
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<{:?}-{:?}>", self.start, self.end)
    }
}

/// Zero-indexed line-column position in source code.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Pos {
    /// The zero-indexed line.
    pub line: usize,
    /// The zero-indexed column.
    pub column: usize,
}

impl Pos {
    /// The line 0, column 0 position.
    pub const ZERO: Self = Self { line: 0, column: 0 };

    /// Create a new position from line and column.
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl Offset for Pos {
    fn offset(self, by: Self) -> Self {
        by + self
    }
}

impl Add for Pos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        if rhs.line == 0 {
            Self {
                line: self.line,
                column: self.column + rhs.column,
            }
        } else {
            Self {
                line: self.line + rhs.line,
                column: rhs.column,
            }
        }
    }
}

impl Sub for Pos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        if self.line == rhs.line {
            Self {
                line: 0,
                column: self.column - rhs.column,
            }
        } else {
            Self {
                line: self.line - rhs.line,
                column: self.column,
            }
        }
    }
}

impl Debug for Pos {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}