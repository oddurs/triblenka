use std::fmt;

/// A byte range in the source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// First byte.
    pub start: usize,
    /// One past the last byte.
    pub end: usize,
}

impl Span {
    /// A span covering `start..end`.
    #[must_use]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// The 1-indexed line and column of this span's start within `source`.
    #[must_use]
    pub fn line_col(&self, source: &str) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for (offset, ch) in source.char_indices() {
            if offset >= self.start {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }
}

/// A node of the parsed template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// Literal markup.
    Text {
        /// The literal text.
        value: String,
        /// Where it came from.
        span: Span,
    },
    /// `{ expr }` — interpolated and escaped.
    Expr {
        /// Source text of the expression, verbatim.
        source: String,
        /// Where it came from.
        span: Span,
        /// True when the interpolation sits inside a tag, and therefore inside an attribute value.
        attribute: bool,
    },
    /// `{#if cond}` … `{:else}` … `{/if}`
    If {
        /// Source of the condition.
        cond: String,
        /// Rendered when the condition holds.
        then: Vec<Node>,
        /// Rendered otherwise; empty when there is no `{:else}`.
        otherwise: Vec<Node>,
        /// Span of the opening tag.
        span: Span,
    },
    /// `{#for binding in seq}` … `{/for}`
    For {
        /// The loop variable.
        binding: String,
        /// Source of the sequence expression.
        seq: String,
        /// The loop body.
        body: Vec<Node>,
        /// Span of the opening tag.
        span: Span,
    },
}

/// A parsed `.tri` file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document {
    /// Raw frontmatter source, if the file opened with a `---` fence.
    pub frontmatter: Option<String>,
    /// Span of the frontmatter body.
    pub frontmatter_span: Option<Span>,
    /// The template body.
    pub nodes: Vec<Node>,
}

/// What went wrong while parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// A `{` was opened and never closed.
    UnclosedExpression,
    /// A block was opened and never closed.
    UnclosedBlock(&'static str),
    /// A closing tag with no matching opening tag.
    UnexpectedClose(String),
    /// `{:else}` outside an `{#if}`.
    StrayElse,
    /// A `{#for}` without `in`.
    MalformedFor,
    /// An empty `{}`.
    EmptyExpression,
    /// The frontmatter fence was opened and never closed.
    UnclosedFrontmatter,
    /// A block name we do not know.
    UnknownBlock(String),
    /// An interpolation in an attribute value that is not quoted.
    UnquotedAttribute,
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnclosedExpression => write!(f, "this `{{` is never closed"),
            Self::UnclosedBlock(name) => write!(f, "`{{#{name}}}` is never closed"),
            Self::UnexpectedClose(name) => {
                write!(f, "`{{/{name}}}` closes a block that was never opened")
            }
            Self::StrayElse => write!(f, "`{{:else}}` outside an `{{#if}}`"),
            Self::MalformedFor => write!(f, "expected `{{#for <name> in <expression>}}`"),
            Self::EmptyExpression => write!(f, "empty expression"),
            Self::UnclosedFrontmatter => write!(f, "frontmatter is never closed with `---`"),
            Self::UnknownBlock(name) => {
                write!(
                    f,
                    "unknown block `{{#{name}}}`; expected one of `if`, `for`"
                )
            }
            Self::UnquotedAttribute => write!(
                f,
                "interpolation into an unquoted attribute value; wrap the value in quotes"
            ),
        }
    }
}

/// A parse failure, with the span that caused it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// What went wrong.
    pub kind: ParseErrorKind,
    /// Where it went wrong.
    pub span: Span,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    /// Render the error against its source, the way the CLI would.
    #[must_use]
    pub fn report(&self, path: &str, source: &str) -> String {
        let (line, col) = self.span.line_col(source);
        let text = source.lines().nth(line - 1).unwrap_or_default();
        format!(
            "error: {}\n  ┌─ {path}:{line}:{col}\n  │\n{line:>3} │ {text}\n  │ {:>width$}^\n",
            self.kind,
            "",
            width = col
        )
    }
}
