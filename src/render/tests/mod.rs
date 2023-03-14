use termcolor::Buffer;
use crate::diagnostic::Severity;
use crate::file::SimpleFile;
use crate::render::color::DefaultColorConfig;
use super::*;

#[test]
fn test_header_1() {
    let mut buf = Buffer::no_color();
    let mut renderer = DiagnosticRenderer::new(&mut buf, DefaultColorConfig,
        SimpleFile::new("main.test", "unused source"),
        RenderConfig { surrounding_lines: 0 });
    renderer.render(vec![
        Diagnostic::new(Severity::Error)
            .with_name("test/diagnostic_1")
            .with_message("Test message")
    ]).unwrap();

    let buf = buf.into_inner();
    let result = String::from_utf8_lossy(&buf);

    assert_eq!(result, "error[test/diagnostic_1]: Test message\n");
}
