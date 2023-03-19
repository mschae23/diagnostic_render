use super::*;

#[test]
fn test_1() {
    let mut buf = Buffer::no_color();
    let file = SimpleFile::new("test_file.test", "test file contents");
    let diagnostic = Diagnostic::new(Severity::Error)
        .with_message("Test message")
        .with_annotation( Annotation::new(AnnotationStyle::Primary, (), 5..9)
            .with_label("test label"));
    let mut renderer = DiagnosticRenderer::new(&mut buf, DefaultColorConfig,
        file, RenderConfig { surrounding_lines: 0 });
    renderer.render(vec![diagnostic]).unwrap();

    let buf = buf.into_inner();
    let result = String::from_utf8_lossy(&buf);

    insta::assert_snapshot!(result);
}

#[test]
fn test_separate_lines_1() {
    let mut buf = Buffer::no_color();
    let file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source);\n");
    let diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error)
        .with_message("Mismatched types")
        .with_annotation(Annotation::new(AnnotationStyle::Primary, (), 3..13)
            .with_label("expected type annotation here"))
        .with_annotation(Annotation::new(AnnotationStyle::Secondary, (), 28..31)
            .with_label("due to this"));
    let mut renderer = DiagnosticRenderer::new(&mut buf, DefaultColorConfig,
        file, RenderConfig { surrounding_lines: 0 });
    renderer.render(vec![diagnostic]).unwrap();

    let buf = buf.into_inner();
    let result = String::from_utf8_lossy(&buf);

    insta::assert_snapshot!(result);
}

#[test]
fn test_same_line_1() {
    let mut buf = Buffer::no_color();
    let file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source);\n");
    let diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error)
        .with_message("Mismatched types")
        .with_annotation(Annotation::new(AnnotationStyle::Primary, (), 11..13)
            .with_label("number"))
        .with_annotation(Annotation::new(AnnotationStyle::Secondary, (), 4..8)
            .with_label("identifier"));
    let mut renderer = DiagnosticRenderer::new(&mut buf, DefaultColorConfig,
        file, RenderConfig { surrounding_lines: 0 });
    renderer.render(vec![diagnostic]).unwrap();

    let buf = buf.into_inner();
    let result = String::from_utf8_lossy(&buf);

    insta::assert_snapshot!(result);
}

#[test]
fn test_overlapping_1() {
    let mut buf = Buffer::no_color();
    let file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source);\n");
    let diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error)
        .with_message("Mismatched types")
        .with_annotation(Annotation::new(AnnotationStyle::Primary, (), 4..13)
            .with_label("something"))
        .with_annotation(Annotation::new(AnnotationStyle::Secondary, (), 8..11)
            .with_label("something else"));
    let mut renderer = DiagnosticRenderer::new(&mut buf, DefaultColorConfig,
        file, RenderConfig { surrounding_lines: 0 });
    renderer.render(vec![diagnostic]).unwrap();

    let buf = buf.into_inner();
    let result = String::from_utf8_lossy(&buf);

    insta::assert_snapshot!(result);
}
