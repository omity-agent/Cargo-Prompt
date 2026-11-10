extern crate alloc;
mod diagnostic;
mod fragment;
mod grouping;
mod output;
use crate::{diagnostic::CargoMessage, grouping::Diagnostics};
use core::{
    error::Error,
    fmt::{self, Display, Formatter},
};
use error_stack::{Report, ResultExt as _};
use regex::RegexSet;
use std::io::{BufRead, Write};
#[derive(Debug)]
pub struct Failure(pub &'static str);
impl Display for Failure {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl Error for Failure {}
#[inline]
pub fn format<R, W>(reader: R, mut writer: W) -> Result<(), Report<Failure>>
where
    R: BufRead,
    W: Write,
{
    let patterns: Vec<String> = sonic_rs::from_str(include_str!("../omissions.json"))
        .change_context(Failure("invalid diagnostic message filters"))?;
    let omissions =
        RegexSet::new(patterns).change_context(Failure("invalid diagnostic message patterns"))?;
    let mut diagnostics = Diagnostics::default();
    for (index, line) in reader.lines().enumerate() {
        let input = line.change_context(Failure("could not read Cargo output"))?;
        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.starts_with('{') {
            eprintln!("{input}");
            continue;
        }
        let event = sonic_rs::from_str::<CargoMessage>(trimmed)
            .change_context(Failure("invalid Cargo JSON message"))
            .attach_with(|| format!("output line index: {index}"))?;
        if let CargoMessage::Diagnostic { message } = event {
            diagnostics.insert(&message, &omissions);
        }
    }
    output::write(&diagnostics, &mut writer)
        .change_context(Failure("could not write the diagnostic report"))?;
    writer
        .flush()
        .change_context(Failure("could not flush the diagnostic report"))
}
