use crate::ast::{Document, Node, ParseError, ParseErrorKind, Span};

/// Parse a `.tri` source file.
///
/// # Errors
///
/// Returns the first structural problem found, with the span that caused it. Expressions are not
/// validated here — their source is recorded verbatim for `rustc` (release) or the bindings (dev).
pub fn parse(source: &str) -> Result<Document, ParseError> {
    let mut parser = Parser {
        source,
        bytes: source.as_bytes(),
        pos: 0,
    };
    let (frontmatter, frontmatter_span) = parser.frontmatter()?;
    let nodes = parser.nodes(None)?;
    Ok(Document {
        frontmatter,
        frontmatter_span,
        nodes,
    })
}

struct Parser<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn frontmatter(&mut self) -> Result<(Option<String>, Option<Span>), ParseError> {
        let rest = &self.source[self.pos..];
        if !rest.starts_with("---") {
            return Ok((None, None));
        }
        let after_fence = self.pos + 3;
        let body_start = match self.source[after_fence..].find('\n') {
            Some(offset) => after_fence + offset + 1,
            None => {
                return Err(ParseError {
                    kind: ParseErrorKind::UnclosedFrontmatter,
                    span: Span::new(self.pos, self.source.len()),
                });
            }
        };
        let Some(offset) = self.source[body_start..].find("\n---") else {
            return Err(ParseError {
                kind: ParseErrorKind::UnclosedFrontmatter,
                span: Span::new(self.pos, self.source.len()),
            });
        };
        let body_end = body_start + offset;
        let close_end = body_end + 4;
        self.pos = match self.source[close_end..].find('\n') {
            Some(nl) => close_end + nl + 1,
            None => self.source.len(),
        };
        Ok((
            Some(self.source[body_start..body_end].to_owned()),
            Some(Span::new(body_start, body_end)),
        ))
    }

    /// Parse nodes until end of input, or until the named closing tag when inside a block.
    fn nodes(&mut self, inside: Option<&str>) -> Result<Vec<Node>, ParseError> {
        let mut nodes = Vec::new();
        let mut text_start = self.pos;
        // Whether the scanner is between `<` and `>`, which is how an interpolation learns that it
        // lands in an attribute value and must escape quotes too.
        let mut in_tag = false;

        while self.pos < self.bytes.len() {
            if self.bytes[self.pos] != b'{' {
                match self.bytes[self.pos] {
                    b'<' => in_tag = true,
                    b'>' => in_tag = false,
                    _ => {}
                }
                self.pos += 1;
                continue;
            }
            let brace = self.pos;
            let next = self.bytes.get(brace + 1).copied();

            // A closing or else tag belongs to our caller.
            if matches!(next, Some(b'/') | Some(b':')) {
                let (name, _end) = self.peek_tag(brace)?;
                let is_close = next == Some(b'/');
                if is_close && inside != Some(name.as_str()) {
                    if inside.is_none() {
                        return Err(ParseError {
                            kind: ParseErrorKind::UnexpectedClose(name),
                            span: Span::new(brace, brace + 2),
                        });
                    }
                    return Err(ParseError {
                        kind: ParseErrorKind::UnclosedBlock(static_name(inside)),
                        span: Span::new(brace, brace + 2),
                    });
                }
                if !is_close && inside != Some("if") {
                    return Err(ParseError {
                        kind: ParseErrorKind::StrayElse,
                        span: Span::new(brace, brace + 2),
                    });
                }
                push_text(&mut nodes, self.source, text_start, brace);
                return Ok(nodes);
            }

            push_text(&mut nodes, self.source, text_start, brace);
            let node = if next == Some(b'#') {
                self.block(brace)?
            } else {
                self.interpolation(brace, in_tag)?
            };
            nodes.push(node);
            text_start = self.pos;
        }

        if let Some(open) = inside {
            return Err(ParseError {
                kind: ParseErrorKind::UnclosedBlock(static_name(Some(open))),
                span: Span::new(self.pos.saturating_sub(1), self.pos),
            });
        }
        push_text(&mut nodes, self.source, text_start, self.source.len());
        Ok(nodes)
    }

    fn interpolation(&mut self, brace: usize, attribute: bool) -> Result<Node, ParseError> {
        let (source, span) = self.take_braced(brace)?;
        let trimmed = source.trim();
        if trimmed.is_empty() {
            return Err(ParseError {
                kind: ParseErrorKind::EmptyExpression,
                span,
            });
        }
        Ok(Node::Expr {
            source: trimmed.to_owned(),
            span,
            attribute,
        })
    }

    fn block(&mut self, brace: usize) -> Result<Node, ParseError> {
        let (inner, span) = self.take_braced(brace)?;
        let inner = inner.trim();
        let body = inner.strip_prefix('#').unwrap_or(inner).trim();
        let (name, rest) = body.split_once(char::is_whitespace).unwrap_or((body, ""));

        match name {
            "if" => {
                let cond = rest.trim().to_owned();
                if cond.is_empty() {
                    return Err(ParseError {
                        kind: ParseErrorKind::EmptyExpression,
                        span,
                    });
                }
                let then = self.nodes(Some("if"))?;
                let mut otherwise = Vec::new();
                // We stopped on either `{:else}` or `{/if}`.
                let tag_start = self.pos;
                let (_tag, _end) = self.peek_tag(tag_start)?;
                if self.bytes.get(tag_start + 1).copied() == Some(b':') {
                    self.skip_tag(tag_start)?;
                    otherwise = self.nodes(Some("if"))?;
                }
                let close = self.pos;
                self.skip_tag(close)?;
                Ok(Node::If {
                    cond,
                    then,
                    otherwise,
                    span,
                })
            }
            "for" => {
                let Some((binding, seq)) = rest.split_once(" in ") else {
                    return Err(ParseError {
                        kind: ParseErrorKind::MalformedFor,
                        span,
                    });
                };
                let binding = binding.trim().to_owned();
                let seq = seq.trim().to_owned();
                if binding.is_empty() || seq.is_empty() {
                    return Err(ParseError {
                        kind: ParseErrorKind::MalformedFor,
                        span,
                    });
                }
                let body = self.nodes(Some("for"))?;
                let close = self.pos;
                self.skip_tag(close)?;
                Ok(Node::For {
                    binding,
                    seq,
                    body,
                    span,
                })
            }
            other => Err(ParseError {
                kind: ParseErrorKind::UnknownBlock(other.to_owned()),
                span,
            }),
        }
    }

    /// Consume `{ … }` starting at `brace`, tracking nested braces so expressions may contain them.
    fn take_braced(&mut self, brace: usize) -> Result<(String, Span), ParseError> {
        let mut depth = 0_usize;
        let mut index = brace;
        while index < self.bytes.len() {
            match self.bytes[index] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        self.pos = index + 1;
                        let span = Span::new(brace, self.pos);
                        return Ok((self.source[brace + 1..index].to_owned(), span));
                    }
                }
                _ => {}
            }
            index += 1;
        }
        Err(ParseError {
            kind: ParseErrorKind::UnclosedExpression,
            span: Span::new(brace, self.source.len()),
        })
    }

    fn peek_tag(&self, brace: usize) -> Result<(String, usize), ParseError> {
        let mut index = brace + 2;
        while index < self.bytes.len() && self.bytes[index] != b'}' {
            index += 1;
        }
        if index >= self.bytes.len() {
            return Err(ParseError {
                kind: ParseErrorKind::UnclosedExpression,
                span: Span::new(brace, self.source.len()),
            });
        }
        Ok((self.source[brace + 2..index].trim().to_owned(), index + 1))
    }

    fn skip_tag(&mut self, brace: usize) -> Result<(), ParseError> {
        let (_, end) = self.peek_tag(brace)?;
        self.pos = end;
        Ok(())
    }
}

fn static_name(inside: Option<&str>) -> &'static str {
    match inside {
        Some("for") => "for",
        _ => "if",
    }
}

fn push_text(nodes: &mut Vec<Node>, source: &str, start: usize, end: usize) {
    if end > start {
        nodes.push(Node::Text {
            value: source[start..end].to_owned(),
            span: Span::new(start, end),
        });
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn nodes(src: &str) -> Vec<Node> {
        parse(src).expect("parses").nodes
    }

    #[test]
    fn plain_text_is_one_node() {
        assert_eq!(
            nodes("<p>hello</p>"),
            vec![Node::Text {
                value: "<p>hello</p>".into(),
                span: Span::new(0, 12)
            }]
        );
    }

    #[test]
    fn interpolation_records_source_and_span() {
        let parsed = nodes("<h1>{ post.title }</h1>");
        assert_eq!(
            parsed[1],
            Node::Expr {
                source: "post.title".into(),
                span: Span::new(4, 18),
                attribute: false
            }
        );
    }

    #[test]
    fn expressions_may_contain_braces() {
        let parsed = nodes("{ posts.iter().map(|p| { p.title }) }");
        assert_eq!(
            parsed[0],
            Node::Expr {
                source: "posts.iter().map(|p| { p.title })".into(),
                span: Span::new(0, 37),
                attribute: false
            }
        );
    }

    #[test]
    fn an_interpolation_inside_a_tag_is_marked_as_an_attribute() {
        let parsed = nodes(r#"<meta content="{ description }"><p>{ body }</p>"#);
        let flags: Vec<bool> = parsed
            .iter()
            .filter_map(|n| match n {
                Node::Expr { attribute, .. } => Some(*attribute),
                _ => None,
            })
            .collect();
        assert_eq!(
            flags,
            vec![true, false],
            "attribute context must be detected"
        );
    }

    #[test]
    fn frontmatter_is_captured_and_skipped() {
        let doc = parse("---\nlet x = 1;\n---\n<p>hi</p>").expect("parses");
        assert_eq!(doc.frontmatter.as_deref(), Some("let x = 1;"));
        assert_eq!(
            doc.nodes,
            vec![Node::Text {
                value: "<p>hi</p>".into(),
                span: Span::new(19, 28)
            }]
        );
    }

    #[test]
    fn if_else_nests() {
        let parsed = nodes("{#if live}<p>yes</p>{:else}<p>no</p>{/if}");
        match &parsed[0] {
            Node::If {
                cond,
                then,
                otherwise,
                ..
            } => {
                assert_eq!(cond, "live");
                assert_eq!(then.len(), 1);
                assert_eq!(otherwise.len(), 1);
            }
            other => panic!("expected an if, got {other:?}"),
        }
    }

    #[test]
    fn for_binds_and_nests() {
        let parsed = nodes("{#for post in posts}<li>{ post.title }</li>{/for}");
        match &parsed[0] {
            Node::For {
                binding, seq, body, ..
            } => {
                assert_eq!(binding, "post");
                assert_eq!(seq, "posts");
                assert_eq!(body.len(), 3);
            }
            other => panic!("expected a for, got {other:?}"),
        }
    }

    #[test]
    fn unclosed_block_reports_a_span() {
        let err = parse("{#if live}<p>yes</p>").expect_err("must fail");
        assert_eq!(err.kind, ParseErrorKind::UnclosedBlock("if"));
    }

    #[test]
    fn stray_close_is_named() {
        let err = parse("<p>x</p>{/if}").expect_err("must fail");
        assert_eq!(err.kind, ParseErrorKind::UnexpectedClose("if".into()));
    }

    #[test]
    fn unknown_block_suggests_the_known_ones() {
        let err = parse("{#while x}{/while}").expect_err("must fail");
        assert_eq!(err.kind, ParseErrorKind::UnknownBlock("while".into()));
    }

    #[test]
    fn malformed_for_is_caught() {
        assert_eq!(
            parse("{#for posts}{/for}").expect_err("must fail").kind,
            ParseErrorKind::MalformedFor
        );
    }

    #[test]
    fn empty_expression_is_caught() {
        assert_eq!(
            parse("<p>{ }</p>").expect_err("must fail").kind,
            ParseErrorKind::EmptyExpression
        );
    }

    #[test]
    fn error_report_points_at_the_source() {
        let src = "<p>ok</p>\n{#if live}\n";
        let err = parse(src).expect_err("must fail");
        let report = err.report("index.tri", src);
        assert!(report.contains("index.tri:"), "{report}");
        assert!(report.contains("never closed"), "{report}");
    }
}
