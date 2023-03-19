use super::*;

#[test]
fn test_simple_1() {
    let mut buf = Buffer::no_color();
    let file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source);\n");
    let diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error)
        .with_message("Some message")
        .with_annotation(Annotation::new(AnnotationStyle::Primary, (), 4..31)
            .with_label("something"));
    let mut renderer = DiagnosticRenderer::new(&mut buf, DefaultColorConfig,
        file, RenderConfig { surrounding_lines: 0 });
    renderer.render(vec![diagnostic]).unwrap();

    let buf = buf.into_inner();
    let result = String::from_utf8_lossy(&buf);

    insta::assert_snapshot!(result);
}

#[test]
fn test_1() {
    let mut buf = Buffer::no_color();
    let file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source);\n");
    let diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error)
        .with_message("Some message")
        .with_annotation(Annotation::new(AnnotationStyle::Primary, (), 11..31)
            .with_label("something"))
        .with_annotation(Annotation::new(AnnotationStyle::Secondary, (), 4..8)
            .with_label("something else"));
    let mut renderer = DiagnosticRenderer::new(&mut buf, DefaultColorConfig,
        file, RenderConfig { surrounding_lines: 0 });
    renderer.render(vec![diagnostic]).unwrap();

    let buf = buf.into_inner();
    let result = String::from_utf8_lossy(&buf);

    insta::assert_snapshot!(result);
}

#[test]
fn test_with_ending_1() {
    let mut buf = Buffer::no_color();
    let file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source);\n");
    let diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error)
        .with_message("Some message")
        .with_annotation(Annotation::new(AnnotationStyle::Primary, (), 28..38)
            .with_label("something"))
        .with_annotation(Annotation::new(AnnotationStyle::Secondary, (), 11..24)
            .with_label("something else"));
    let mut renderer = DiagnosticRenderer::new(&mut buf, DefaultColorConfig,
        file, RenderConfig { surrounding_lines: 0 });
    renderer.render(vec![diagnostic]).unwrap();

    let buf = buf.into_inner();
    let result = String::from_utf8_lossy(&buf);

    insta::assert_snapshot!(result);
}
