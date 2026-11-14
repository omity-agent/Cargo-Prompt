use cargo_prompt::format;
fn render(input: &str) -> String {
    let mut bytes = Vec::new();
    format(input.as_bytes(), &mut bytes).unwrap();
    String::from_utf8(bytes).unwrap()
}
#[test]
fn groups_diagnostics_without_losing_local_context() {
    let output = render(&include_str!("diagnostic-records.jsonl").repeat(2));
    assert_eq!(output.matches("<Warning>").count(), 1);
    assert_eq!(output.matches("<example>").count(), 1);
    assert_eq!(output.matches("</example>").count(), 1);
    assert_eq!(output.matches("<Snippet ").count(), 3);
    assert_eq!(output.matches("<src/").count(), 3);
    assert!(output.contains("<Message>use <T> & \"value\"</Message>"));
    assert!(output.contains("<Snippet start=\"1:3\" end=\"1:8\">"));
    assert!(
        output.contains("<Highlight start=\"1:3\" end=\"1:8\">  let value = \"<T>&\";</Highlight>")
    );
    assert!(output.contains(include_str!("highlight-blocks.xml")));
    assert_eq!(output.matches("<Highlight ").count(), 4);
    assert_eq!(output.matches("<Label>secondary</Label>").count(), 1);
    let beta = output.split("<src/beta.rs>").nth(1).unwrap();
    let beta_end = beta.split("</src/beta.rs>").next().unwrap();
    for (text, detail) in [
        (beta_end, "<Detail>replace <value></Detail>"),
        (beta_end, "<Detail>local explanation</Detail>"),
        (
            output.as_str(),
            "<Detail>general <T> & explanation</Detail>",
        ),
    ] {
        assert_eq!(text.matches(detail).count(), 1);
    }
    assert!(!beta_end.contains("drop(&value)"));
}
#[test]
fn handles_non_diagnostics_and_rejects_invalid_json() {
    const FINISHED: &[u8] = b"{\"reason\":\"build-finished\",\"success\":true}\n";
    let mut output = Vec::new();
    format(FINISHED, &mut output).unwrap();
    assert_eq!(output.as_slice(), b"");
    assert!(format(b"{invalid}\n".as_slice(), &mut output).is_err());
}
#[test]
fn filters_boilerplate_while_preserving_sources_and_descendants() {
    let output = render(include_str!("boilerplate.jsonl"));
    let messages = output
        .lines()
        .filter(|line| line.starts_with("<Detail") || line.starts_with("<Message>"))
        .collect::<Vec<_>>()
        .join("\n");
    for omitted in [
        "`-W clippy::sample` implied by `-W clippy::all`",
        "to override",
        "`-W lint` implied by `-W group`",
        "for a full list of items that can appear in modules, see",
    ] {
        assert!(!messages.contains(omitted), "{messages}");
    }
    assert!(output.contains("let guide = \"for further information visit"));
    assert!(output.contains("<Label>`label` implied by `source`</Label>"));
    assert!(messages.contains("useful nested detail"));
    assert!(messages.contains("useful standalone detail"));
    assert!(messages.contains("on by default"));
    assert!(messages.contains("but keep this context"));
    assert!(output.contains("<src/hint.rs>"));
}
#[test]
fn keeps_errors_and_notes_without_replacement_output() {
    let output = render(include_str!("protected.jsonl"));
    assert!(output.contains("<Message>`a` implied by `b`</Message>"));
    assert!(output.contains("<Message>try</Message>"));
    assert!(output.contains("<Detail code=\"custom\">try</Detail>"));
    assert!(output.contains("<Detail>retained explanation</Detail>"));
    assert!(!output.contains("<Detail>try</Detail>"));
    assert!(!output.contains("<Replacement"));
}
