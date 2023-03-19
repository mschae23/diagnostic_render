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

    insta::assert_snapshot!(result, @r###"
    error[test/diagnostic_1]: Test message
    "###);
}

/* // TODO this completely breaks the renderer right now
#[test]
fn test_fibonacci() {
    let source = r#"
    pub fn fibonacci(n: i32) -> u64 {
        if n < 0 {
            panic!("{} is negative!", n);
        } else if n == 0 {
            panic!("zero is not a right argument to fibonacci()!");
        } else if n == 1 {
            return 1;
        }

        let mut sum = 0;
        let mut last = 0;
        let mut curr = 1;
        for _i in 1..n {
            sum = last + curr;
            last = curr;
            curr = sum;
        }
        sum
    }
    "#;

    let mut buf = Buffer::no_color();
    let file = SimpleFile::new("test_file.test", source);

    let mut diagnostic = Diagnostic::new(Severity::Note)
        .with_message("A fibonacci function");

    {
        let mut opened = Vec::new();

        for (i, b) in source.bytes().enumerate() {
            let c = char::from(b);

            match c {
                '(' | '[' | '{' => opened.push((i, c)),
                '"' if opened.is_empty() || opened.last().unwrap().1 != '"' => {
                    opened.push((i, c));
                },
                ')' | ']' | '}' | '"' => {
                    if let Some((start, expected)) = opened.pop() {
                        if c == expected {
                            let range = start..i;
                            let label = match c {
                                ')' => "this is a pair of parenthesis",
                                ']' => "this is a pair of brackets",
                                '}' => "this is a pair of braces",
                                _ => "this is a string",
                            };
                            diagnostic = diagnostic.with_annotation(Annotation::new(AnnotationStyle::Secondary, (), range)
                                    .with_label(label));
                        }
                    }
                },
                _ => {},
            }
        }

        diagnostic = diagnostic.with_annotation(Annotation::new(AnnotationStyle::Primary, (), 0..source.len())
            .with_label("this is the whole program"));
    }

    let mut renderer = DiagnosticRenderer::new(&mut buf, DefaultColorConfig,
        file, RenderConfig { surrounding_lines: 0 });
    renderer.render(vec![diagnostic]).unwrap();

    let buf = buf.into_inner();
    let result = String::from_utf8_lossy(&buf);

    insta::assert_snapshot!(result);
}
*/

mod singleline;
mod ending;
mod starting;
