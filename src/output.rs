use crate::{
    diagnostic::{Expansion, SourceLine, Span},
    fragment::{Annotations, Detail, Excerpt},
    grouping::Diagnostics,
};
use alloc::collections::BTreeSet;
use std::io::{self, Write};
fn detail(writer: &mut impl Write, info: &Detail) -> io::Result<()> {
    write!(writer, "<Detail")?;
    if let Some(code) = info.code.as_ref() {
        write!(writer, " code=\"{code}\"")?;
    }
    writeln!(writer, ">{}</Detail>", info.message)
}
fn source(
    writer: &mut impl Write,
    start: usize,
    text: &[SourceLine],
    highlighted: bool,
) -> io::Result<()> {
    for (offset, line) in text.iter().enumerate() {
        let number = start
            .checked_add(offset)
            .ok_or_else(|| io::Error::other("source line number overflow"))?;
        if highlighted {
            writeln!(
                writer,
                "<Highlight start=\"{number}:{}\" end=\"{number}:{}\">{}</Highlight>",
                line.highlight_start, line.highlight_end, line.text
            )?;
        } else {
            writeln!(writer, "<Line number=\"{number}\">{}</Line>", line.text)?;
        }
    }
    Ok(())
}
fn location(writer: &mut impl Write, tag: &str, span: &Span) -> io::Result<()> {
    writeln!(
        writer,
        "<{tag} path=\"{}\" start=\"{}:{}\" end=\"{}:{}\">",
        span.file_name, span.line_start, span.column_start, span.line_end, span.column_end
    )?;
    source(writer, span.line_start, &span.text, tag != "CallSite")?;
    expansion(writer, span.expansion.as_deref())?;
    writeln!(writer, "</{tag}>")
}
fn expansion(writer: &mut impl Write, source: Option<&Expansion>) -> io::Result<()> {
    if let Some(origin) = source {
        writeln!(writer, "<Expansion macro=\"{}\">", origin.macro_decl_name)?;
        location(writer, "CallSite", &origin.span)?;
        if let Some(definition) = origin.def_site_span.as_ref() {
            location(writer, "Definition", definition)?;
        }
        writeln!(writer, "</Expansion>")?;
    }
    Ok(())
}
fn snippet(
    writer: &mut impl Write,
    excerpt: &Excerpt,
    annotations: &Annotations,
    notes: &BTreeSet<Detail>,
) -> io::Result<()> {
    writeln!(
        writer,
        "<Snippet start=\"{}:{}\" end=\"{}:{}\">",
        excerpt.line_start, excerpt.column_start, excerpt.line_end, excerpt.column_end
    )?;
    source(writer, excerpt.line_start, &excerpt.text, true)?;
    for label in &annotations.labels {
        writeln!(writer, "<Label>{label}</Label>")?;
    }
    for info in annotations.details.difference(notes) {
        detail(writer, info)?;
    }
    expansion(writer, excerpt.expansion.as_deref())?;
    writeln!(writer, "</Snippet>")
}
pub fn write<W>(diagnostics: &Diagnostics, writer: &mut W) -> io::Result<()>
where
    W: Write,
{
    for (severity, problems) in &diagnostics.0 {
        let tag = if severity.tag == "Other" {
            severity.level.as_str()
        } else {
            severity.tag
        };
        writeln!(writer, "<{tag}>")?;
        for (issue, problem) in problems {
            let code = issue.code.as_deref().unwrap_or("Issue");
            let issue_tag = code.strip_prefix("clippy::").unwrap_or(code);
            writeln!(writer, "<{issue_tag}>")?;
            writeln!(writer, "<Message>{}</Message>", issue.message)?;
            for note in &problem.notes {
                detail(writer, note)?;
            }
            for (path, snippets) in &problem.files {
                writeln!(writer, "<{path}>")?;
                for (excerpt, annotations) in snippets {
                    snippet(writer, excerpt, annotations, &problem.notes)?;
                }
                writeln!(writer, "</{path}>")?;
            }
            writeln!(writer, "</{issue_tag}>")?;
        }
        writeln!(writer, "</{tag}>")?;
    }
    Ok(())
}
