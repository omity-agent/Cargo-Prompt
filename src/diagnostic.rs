use serde::Deserialize;
#[derive(Debug, Deserialize)]
#[serde(tag = "reason")]
pub enum CargoMessage {
    #[serde(rename = "compiler-message")]
    Diagnostic { message: Diagnostic },
    #[serde(other)]
    Other,
}
#[derive(Debug, Deserialize)]
pub struct Diagnostic {
    pub message: String,
    pub code: Option<DiagnosticCode>,
    pub level: String,
    pub spans: Vec<Span>,
    pub children: Vec<Self>,
}
#[derive(Debug, Deserialize)]
pub struct DiagnosticCode {
    pub code: String,
}
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct Span {
    pub line_start: usize,
    pub column_start: usize,
    pub line_end: usize,
    pub column_end: usize,
    pub file_name: String,
    pub text: Vec<SourceLine>,
    pub label: Option<String>,
    pub suggested_replacement: Option<String>,
    pub expansion: Option<Box<Expansion>>,
}
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct SourceLine {
    pub text: String,
    pub highlight_start: usize,
    pub highlight_end: usize,
}
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct Expansion {
    pub span: Span,
    pub macro_decl_name: String,
    pub def_site_span: Option<Span>,
}
