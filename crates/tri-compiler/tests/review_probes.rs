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
