use alloc::collections::BTreeMap;
use cargo_prompt::Failure;
use clap_lex::{ParsedArg, RawArgs};
use error_stack::{Report, ResultExt as _};
use serde::Deserialize;
use std::ffi::OsString;
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Defaults {
    arguments: Vec<String>,
    format_arguments: Vec<String>,
    environment: BTreeMap<String, String>,
}
impl Defaults {
    fn load() -> Result<Self, Report<Failure>> {
        let configuration: Self = sonic_rs::from_str(include_str!("../invocation.json"))
            .change_context(Failure("invalid Clippy configuration"))?;
        let formats = configuration
            .arguments
            .iter()
            .filter(|argument| argument.starts_with("--message-format"))
            .count();
        let has_json = configuration
            .arguments
            .windows(2)
            .any(|pair| pair == ["--message-format", "json"]);
        if formats != 1 || !has_json {
            return Err(Report::new(Failure(
                "configuration must specify exactly --message-format json",
            )));
        }
        Ok(configuration)
    }
}
#[derive(Debug)]
pub struct Invocation {
    pub arguments: Vec<OsString>,
    pub format_arguments: Vec<OsString>,
    pub environment: BTreeMap<String, String>,
}
fn is_value(argument: &ParsedArg<'_>) -> bool {
    !argument.is_escape() && argument.to_long().is_none() && argument.to_short().is_none()
}
fn fixed_options(arguments: &RawArgs) -> Result<BTreeMap<&str, bool>, Report<Failure>> {
    let mut cursor = arguments.cursor();
    let mut options = BTreeMap::new();
    while let Some(argument) = arguments.next(&mut cursor) {
        let Some((Ok(name), inline_value)) = argument.to_long() else {
            return Err(Report::new(Failure(
                "invocation.json must contain long Cargo options and their values",
            )));
        };
        let separated_value =
            inline_value.is_none() && arguments.peek(&cursor).is_some_and(|next| is_value(&next));
        let takes_value = inline_value.is_some() || separated_value;
        if separated_value {
            let _value = arguments.next_os(&mut cursor);
        }
        if options.insert(name, takes_value).is_some() {
            return Err(
                Report::new(Failure("duplicate fixed Cargo option")).attach(name.to_owned())
            );
        }
    }
    Ok(options)
}
fn format_arguments(defaults: &[String], clippy: &[OsString]) -> Vec<OsString> {
    let raw = RawArgs::new(clippy);
    let mut cursor = raw.cursor();
    let mut arguments: Vec<OsString> = defaults.iter().map(OsString::from).collect();
    while let Some(argument) = raw.next(&mut cursor) {
        if argument.is_escape() {
            break;
        }
        if let Some((Ok("manifest-path"), inline_value)) = argument.to_long() {
            arguments.push(argument.to_value_os().to_owned());
            if inline_value.is_none()
                && let Some(path) = raw.next_os(&mut cursor)
            {
                arguments.push(path.to_owned());
            }
        }
    }
    arguments
}
pub fn invocation(incoming: Vec<OsString>) -> Result<Invocation, Report<Failure>> {
    let configuration = Defaults::load()?;
    let configured = RawArgs::new(&configuration.arguments);
    let fixed = fixed_options(&configured)?;
    let provided = RawArgs::new(incoming);
    let mut cursor = provided.cursor();
    let mut combined: Vec<OsString> = configuration.arguments.iter().map(OsString::from).collect();
    while let Some(argument) = provided.next(&mut cursor) {
        if argument.is_escape() {
            combined.push(argument.to_value_os().to_owned());
            combined.extend(provided.remaining(&mut cursor).map(OsString::from));
            break;
        }
        if let Some((Ok(name), inline_value)) = argument.to_long()
            && let Some(&takes_value) = fixed.get(name)
        {
            if takes_value
                && inline_value.is_none()
                && provided.peek(&cursor).is_some_and(|next| is_value(&next))
            {
                let _value = provided.next_os(&mut cursor);
            }
            continue;
        }
        combined.push(argument.to_value_os().to_owned());
    }
    Ok(Invocation {
        format_arguments: format_arguments(&configuration.format_arguments, &combined),
        arguments: combined,
        environment: configuration.environment,
    })
}
