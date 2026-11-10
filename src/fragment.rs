use crate::diagnostic::{Diagnostic, Expansion, SourceLine, Span};
use alloc::collections::BTreeSet;
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Detail {
    pub code: Option<String>,
    pub message: String,
}
impl From<&Diagnostic> for Detail {
    fn from(diagnostic: &Diagnostic) -> Self {
        Self {
            code: diagnostic.code.as_ref().map(|code| code.code.clone()),
            message: diagnostic.message.clone(),
        }
    }
}
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Excerpt {
    pub line_start: usize,
    pub column_start: usize,
    pub line_end: usize,
    pub column_end: usize,
    pub text: Vec<SourceLine>,
    pub expansion: Option<Box<Expansion>>,
}
impl From<&Span> for Excerpt {
    fn from(span: &Span) -> Self {
        Self {
            line_start: span.line_start,
            column_start: span.column_start,
            line_end: span.line_end,
            column_end: span.column_end,
            text: span.text.clone(),
            expansion: span.expansion.clone(),
        }
    }
}
#[derive(Debug, Default)]
pub struct Annotations {
    pub labels: BTreeSet<String>,
    pub details: BTreeSet<Detail>,
}
impl Annotations {
    pub fn record(&mut self, span: &Span, context: &[Detail]) {
        self.labels.extend(span.label.iter().cloned());
        self.details.extend(context.iter().cloned());
    }
}
