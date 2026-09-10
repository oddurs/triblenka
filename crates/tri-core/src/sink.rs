use crate::Result;
use std::fmt::Write as _;

/// A destination for rendered bytes.
///
/// Object-safe on purpose: the descriptor walker holds `&mut dyn Sink`, so one walk can serve a
/// string build, a streaming body, or a hash.
pub trait Sink {
    /// Write markup through without escaping. Callers must have produced it themselves.
    fn raw(&mut self, s: &str) -> Result<()>;
}

/// `escaped` for every sink, sized or not.
///
/// Kept off [`Sink`] itself so the trait stays object-safe: the walker holds `&mut dyn Sink`, while
/// callers usually hold a concrete one, and both need the method.
pub trait SinkExt {
    /// Write a value, escaped for text content.
    ///
    /// # Errors
    ///
    /// Propagates whatever the underlying sink reports.
    fn escaped(&mut self, value: &dyn Render) -> Result<()>;
}

// No overlap: the blanket impl carries an implicit `Sized` bound, and `dyn Sink` is not sized.
impl<S: Sink> SinkExt for S {
    fn escaped(&mut self, value: &dyn Render) -> Result<()> {
        value.render_to(self)
    }
}

impl SinkExt for dyn Sink + '_ {
    fn escaped(&mut self, value: &dyn Render) -> Result<()> {
        value.render_to(self)
    }
}

/// A value that can be rendered into a [`Sink`].
pub trait Render {
    /// Write `self`, escaping as appropriate for text content.
    fn render_to(&self, sink: &mut dyn Sink) -> Result<()>;
}

/// Markup that is already safe and must not be escaped.
///
/// Raw output is deliberately a type rather than a directive: it is greppable, and it cannot be
/// reached by accident.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Html<T>(pub T);

impl<T: AsRef<str>> Render for Html<T> {
    fn render_to(&self, sink: &mut dyn Sink) -> Result<()> {
        sink.raw(self.0.as_ref())
    }
}

impl Render for str {
    fn render_to(&self, sink: &mut dyn Sink) -> Result<()> {
        sink.raw(&escape_text(self))
    }
}

impl Render for String {
    fn render_to(&self, sink: &mut dyn Sink) -> Result<()> {
        self.as_str().render_to(sink)
    }
}

impl<T: Render + ?Sized> Render for &T {
    fn render_to(&self, sink: &mut dyn Sink) -> Result<()> {
        (**self).render_to(sink)
    }
}

impl<T: Render> Render for Option<T> {
    /// `None` renders nothing at all, rather than the string "None".
    fn render_to(&self, sink: &mut dyn Sink) -> Result<()> {
        match self {
            Some(v) => v.render_to(sink),
            None => Ok(()),
        }
    }
}

macro_rules! render_via_display {
    ($($t:ty),*) => {$(
        impl Render for $t {
            fn render_to(&self, sink: &mut dyn Sink) -> Result<()> {
                let mut buf = String::new();
                write!(buf, "{self}")?;
                sink.raw(&buf)
            }
        }
    )*};
}
render_via_display!(
    u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, bool, char
);

/// Escape a string for use in text content.
#[must_use]
pub fn escape_text(s: &str) -> String {
    escape_with(s, false)
}

/// Escape a string for use inside a quoted attribute value.
#[must_use]
pub fn escape_attr(s: &str) -> String {
    escape_with(s, true)
}

fn escape_with(s: &str, attribute: bool) -> String {
    let needs = s
        .bytes()
        .any(|b| matches!(b, b'&' | b'<' | b'>') || (attribute && matches!(b, b'"' | b'\'')));
    if !needs {
        return s.to_owned();
    }
    let mut out = String::with_capacity(s.len() + 16);
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' if attribute => out.push_str("&quot;"),
            '\'' if attribute => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

/// A sink that builds a `String`.
///
/// Size hints matter: a page re-rendered with a hint from its previous render allocates once.
#[derive(Debug, Default)]
pub struct StringSink {
    buf: String,
}

impl StringSink {
    /// An empty sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// An empty sink with room for `capacity` bytes.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: String::with_capacity(capacity),
        }
    }

    /// The bytes written so far.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.buf
    }

    /// Consume the sink and return what was written.
    #[must_use]
    pub fn into_string(self) -> String {
        self.buf
    }

    /// How many bytes were written — the size hint for the next render of the same page.
    #[must_use]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Whether anything has been written.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

impl Sink for StringSink {
    fn raw(&mut self, s: &str) -> Result<()> {
        self.buf.push_str(s);
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn render(v: &dyn Render) -> String {
        let mut sink = StringSink::new();
        sink.escaped(v).unwrap();
        sink.into_string()
    }

    #[test]
    fn escapes_text_but_not_quotes() {
        assert_eq!(
            escape_text(r#"a & b < c > d " e"#),
            r#"a &amp; b &lt; c &gt; d " e"#
        );
    }

    #[test]
    fn escapes_quotes_in_attributes() {
        assert_eq!(escape_attr(r#"a "b" 'c'"#), "a &quot;b&quot; &#39;c&#39;");
    }

    #[test]
    fn leaves_clean_strings_untouched() {
        assert_eq!(escape_text("plain text"), "plain text");
    }

    #[test]
    fn html_is_not_escaped() {
        assert_eq!(render(&Html("<em>hi</em>")), "<em>hi</em>");
        assert_eq!(render(&"<em>hi</em>"), "&lt;em&gt;hi&lt;/em&gt;");
    }

    #[test]
    fn none_renders_nothing() {
        let empty: Option<&str> = None;
        assert_eq!(render(&empty), "");
        assert_eq!(render(&Some("x")), "x");
    }

    #[test]
    fn numbers_render_via_display() {
        assert_eq!(render(&42_u32), "42");
        assert_eq!(render(&true), "true");
    }
}
