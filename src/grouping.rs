use crate::{
    diagnostic::{Diagnostic, Span},
    fragment::{Annotations, Detail, Excerpt},
};
use alloc::collections::{BTreeMap, BTreeSet};
use regex::RegexSet;
fn is_omitted(diagnostic: &Diagnostic, omissions: &RegexSet) -> bool {
    matches!(diagnostic.level.as_str(), "note" | "help")
        && diagnostic.code.is_none()
        && (diagnostic.message == "try"
            || (diagnostic
                .spans
                .iter()
                .all(|span| span.suggested_replacement.is_none())
                && omissions.is_match(&diagnostic.message)))
}
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Severity {
    pub rank: u8,
    pub tag: &'static str,
    pub level: String,
}
impl Severity {
    pub fn from_level(level: &str) -> Self {
        let (rank, tag) = match level {
            "ice" => (0, "Ice"),
            "error" => (1, "Error"),
            "warning" => (2, "Warning"),
            "failure-note" => (3, "FailureNote"),
            "note" => (4, "Note"),
            "help" => (5, "Help"),
            _ => (6, "Other"),
        };
        Self {
            rank,
            tag,
            level: level.to_owned(),
        }
    }
}
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Issue {
    pub code: Option<String>,
    pub message: String,
}
#[derive(Debug, Default)]
pub struct Problem {
    pub notes: BTreeSet<Detail>,
    pub files: BTreeMap<String, BTreeMap<Excerpt, Annotations>>,
}
impl Problem {
    fn record(&mut self, spans: &[Span], context: &[Detail]) {
        if spans.is_empty() {
            self.notes.extend(context.iter().cloned());
        }
        for span in spans {
            self.files
                .entry(span.file_name.clone())
                .or_default()
                .entry(Excerpt::from(span))
                .or_default()
                .record(span, context);
        }
    }
    fn visit(
        &mut self,
        diagnostic: &Diagnostic,
        inherited: &[Span],
        ancestors: &[Detail],
        omissions: &RegexSet,
    ) {
        let spans = if diagnostic.spans.is_empty() {
            inherited
        } else {
            &diagnostic.spans
        };
        let mut context = ancestors.to_vec();
        if !is_omitted(diagnostic, omissions) {
            context.push(Detail::from(diagnostic));
            self.record(spans, &context);
        }
        for child in &diagnostic.children {
            self.visit(child, spans, &context, omissions);
        }
    }
}
#[derive(Debug, Default)]
pub struct Diagnostics(pub BTreeMap<Severity, BTreeMap<Issue, Problem>>);
impl Diagnostics {
    pub(super) fn insert(&mut self, diagnostic: &Diagnostic, omissions: &RegexSet) {
        if is_omitted(diagnostic, omissions) {
            for child in &diagnostic.children {
                self.insert(child, omissions);
            }
            return;
        }
        let issue = Issue {
            code: diagnostic.code.as_ref().map(|code| code.code.clone()),
            message: diagnostic.message.clone(),
        };
        let problem = self
            .0
            .entry(Severity::from_level(&diagnostic.level))
            .or_default()
            .entry(issue)
            .or_default();
        problem.record(&diagnostic.spans, &[]);
        for child in &diagnostic.children {
            problem.visit(child, &[], &[], omissions);
        }
    }
}
