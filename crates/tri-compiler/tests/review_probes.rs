//! Probes written during the M0 review of the parser and the escaping rules.

use tri_compiler::parse;
use tri_core::{Bindings, Html, Result, Sink, SinkExt as _, StringSink, render};

struct One(&'static str);
impl Bindings for One {
    fn value(&self, _index: usize, sink: &mut dyn Sink) -> Result<()> {
        sink.escaped(&self.0)
    }
}

/// Known limitation, filed rather than fixed: the scanner counts braces without knowing about
/// string literals, so a `}` inside a string ends the expression early — silently.
#[test]
fn known_limitation_a_brace_inside_a_string_literal_truncates_the_expression() {
    let document = parse(r#"{ format!("}") }"#).expect("parses");
    let rendered = format!("{:?}", document.nodes);
    assert!(
        rendered.contains(r#"format!(\""#),
        "if this now parses correctly, the limitation is fixed — delete this probe: {rendered}"
    );
}

#[test]
fn an_attribute_interpolation_escapes_quotes() {
    // The fixture's own page template does exactly this:
    //     <meta name="description" content="{ post.description }">
    let doc = parse(r#"<meta content="{ description }">"#).expect("parses");
    let template = tri_compiler::descriptor::lower(&doc);
    let mut sink = StringSink::new();
    render(&template, &One(r#"a " b"#), &mut sink).expect("renders");

    let html = sink.into_string();
    assert_eq!(html, r#"<meta content="a &quot; b">"#);
}

#[test]
fn attribute_escaping_does_not_double_escape_ampersands() {
    let doc = parse(r#"<a title="{ text }">x</a>"#).expect("parses");
    let template = tri_compiler::descriptor::lower(&doc);
    let mut sink = StringSink::new();
    render(&template, &One("a & b"), &mut sink).expect("renders");
    assert_eq!(sink.into_string(), r#"<a title="a &amp; b">x</a>"#);
}

#[test]
fn text_context_still_leaves_quotes_alone() {
    let doc = parse("<p>{ text }</p>").expect("parses");
    let template = tri_compiler::descriptor::lower(&doc);
    let mut sink = StringSink::new();
    render(&template, &One(r#"it's "fine""#), &mut sink).expect("renders");
    assert_eq!(sink.into_string(), r#"<p>it's "fine"</p>"#);
}

#[test]
fn probe_html_newtype_still_bypasses_escaping_as_designed() {
    let mut sink = StringSink::new();
    sink.escaped(&Html("<em>ok</em>")).expect("renders");
    assert_eq!(sink.into_string(), "<em>ok</em>");
}

struct Always(&'static str);
impl Bindings for Always {
    fn value(&self, _index: usize, sink: &mut dyn Sink) -> Result<()> {
        sink.escaped(&self.0)
    }
    fn truthy(&self, _index: usize) -> bool {
        true
    }
}

const PAYLOAD: &str = r#"a" onload="evil()"#;

fn render_str(source: &str, value: &'static str) -> String {
    let doc = parse(source).expect("parses");
    let template = tri_compiler::descriptor::lower(&doc);
    let mut sink = StringSink::new();
    render(&template, &Always(value), &mut sink).expect("renders");
    sink.into_string()
}

#[test]
fn attribute_context_survives_a_block_body() {
    let html = render_str(r#"<a title="{#if c}{ x }{/if}">y</a>"#, PAYLOAD);
    assert!(!html.contains(r#"onload="evil()""#), "{html}");
    assert!(html.contains("&quot;"), "{html}");
}

#[test]
fn a_literal_gt_inside_an_attribute_does_not_end_tag_context() {
    let html = render_str(r#"<a data-r="a > b" title="{ x }">y</a>"#, PAYLOAD);
    assert!(!html.contains(r#"onload="evil()""#), "{html}");
}

#[test]
fn a_literal_lt_in_prose_does_not_start_tag_context() {
    let html = render_str("<p>1 &lt; 2 { x }</p>", "it's fine");
    assert_eq!(html, "<p>1 &lt; 2 it's fine</p>");
}

#[test]
fn interpolation_into_an_unquoted_attribute_is_rejected() {
    let error = parse(r#"<a title={ x }>y</a>"#).expect_err("must be rejected");
    assert_eq!(error.kind, tri_compiler::ParseErrorKind::UnquotedAttribute);
    assert!(
        format!("{error}").contains("wrap the value in quotes"),
        "{error}"
    );
}

#[test]
fn single_quoted_attributes_are_tracked_too() {
    let html = render_str("<a title='{ x }'>y</a>", "it's");
    assert_eq!(html, "<a title='it&#39;s'>y</a>");
}
