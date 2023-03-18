use pretty_assertions::{assert_eq, assert_ne};
use super::*;

#[test]
fn test_1() {
    let file = SimpleFile::new("test_file.test", "test file contents");
    let diagnostic = Diagnostic::new(Severity::Error);
    let annotation = Annotation::new(AnnotationStyle::Primary, (), 5..9)
        .with_label("test label");

    // 1 | test file contents
    //   |      ^^^^ test label

    assert_eq!(calculate(&diagnostic, &file, (), 0, &[&annotation], &[]).unwrap(), vec![
        vec![
            AnnotationData::Start(StartAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 5),
            }),
            AnnotationData::ConnectingSingleline(ConnectingSinglelineAnnotationData {
                style: AnnotationStyle::Primary,
                as_multiline: false,
                severity: Severity::Error,
                line_index: 0,
                start_column_index: 5,
                end_column_index: 9,
            }),
            AnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 9),
            }),
            AnnotationData::Label(LabelAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 10),
                label: String::from("test label"),
            }),
        ],
    ]);
}

#[test]
fn test_separate_lines_1() {
    let file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source);\n");
    let diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

    let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 3..13)
        .with_label("expected type annotation here");
    let annotation2 = Annotation::new(AnnotationStyle::Secondary, (), 28..31)
        .with_label("due to this");

    // 1 | let main = 23;
    //   |    ^^^^^^^^^^ expected type annotation here
    // 2 | something += 3.0;
    //   |              --- due to this

    assert_eq!(calculate(&diagnostic, &file, (), 0, &[&annotation1], &[]).unwrap(), vec![
        vec![
            AnnotationData::Start(StartAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 3),
            }),
            AnnotationData::ConnectingSingleline(ConnectingSinglelineAnnotationData {
                style: AnnotationStyle::Primary,
                as_multiline: false,
                severity: Severity::Error,
                line_index: 0, start_column_index: 3, end_column_index: 13,
            }),
            AnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 13),
            }),
            AnnotationData::Label(LabelAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 14),
                label: String::from("expected type annotation here"),
            }),
        ],
    ]);

    assert_eq!(calculate(&diagnostic, &file, (), 1, &[&annotation2], &[]).unwrap(), vec![
        vec![
            AnnotationData::Start(StartAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(1, 13),
            }),
            AnnotationData::ConnectingSingleline(ConnectingSinglelineAnnotationData {
                style: AnnotationStyle::Secondary,
                as_multiline: false,
                severity: Severity::Error,
                line_index: 1, start_column_index: 13, end_column_index: 16,
            }),
            AnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(1, 16),
            }),
            AnnotationData::Label(LabelAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(1, 17),
                label: String::from("due to this"),
            }),
        ],
    ]);
}

#[test]
fn test_same_line_1() {
    let file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source);\n");
    let diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

    let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 11..13)
        .with_label("number");
    let annotation2 = Annotation::new(AnnotationStyle::Secondary, (), 4..8)
        .with_label("identifier");

    // 1 | let main = 23;
    //   |     ----   ^^ number
    //   |     |
    //   |     identifier

    assert_eq!(calculate(&diagnostic, &file, (), 0, &[&annotation2, &annotation1], &[]).unwrap(), vec![
        vec![
            // First underline (secondary, annotation2)
            AnnotationData::Start(StartAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 4),
            }),
            AnnotationData::ConnectingSingleline(ConnectingSinglelineAnnotationData {
                style: AnnotationStyle::Secondary,
                as_multiline: false,
                severity: Severity::Error,
                line_index: 0, start_column_index: 4, end_column_index: 8,
            }),
            AnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 8),
            }),
            // Second underline (primary, annotation1)
            AnnotationData::Start(StartAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 11),
            }),
            AnnotationData::ConnectingSingleline(ConnectingSinglelineAnnotationData {
                style: AnnotationStyle::Primary,
                as_multiline: false,
                severity: Severity::Error,
                line_index: 0, start_column_index: 11, end_column_index: 13
            }),
            AnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 13),
            }),
            // Label for primary annotation (annotation1)
            AnnotationData::Label(LabelAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 14),
                label: String::from("number"),
            }),
        ],
        // Label for secondary annotation (annotation2)
        // Takes two lines because of the "|" in between the underline and label
        vec![
            AnnotationData::Hanging(HangingAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 4),
            })
        ],
        vec![
            AnnotationData::Label(LabelAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 4),
                label: String::from("identifier"),
            })
        ],
    ]);
}

#[test]
fn test_overlapping_1() {
    let file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source);\n");
    let diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

    let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 4..13)
        .with_label("something");
    let annotation2 = Annotation::new(AnnotationStyle::Secondary, (), 8..11)
        .with_label("something else");

    // 1 | let main = 23;
    //   |     ^^^^---^^
    //   |     |   |
    //   |     |   something else
    //   |     something

    assert_eq!(calculate(&diagnostic, &file, (), 0, &[&annotation2, &annotation1], &[]).unwrap(), vec![
        vec![
            AnnotationData::Start(StartAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 4),
            }),
            AnnotationData::ConnectingSingleline(ConnectingSinglelineAnnotationData {
                style: AnnotationStyle::Primary,
                as_multiline: false,
                severity: Severity::Error,
                line_index: 0,
                start_column_index: 4,
                end_column_index: 13,
            }),
            AnnotationData::Start(StartAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 8),
            }),
            AnnotationData::ConnectingSingleline(ConnectingSinglelineAnnotationData {
                style: AnnotationStyle::Secondary,
                as_multiline: false,
                severity: Severity::Error,
                line_index: 0,
                start_column_index: 8,
                end_column_index: 11,
            }),
            AnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 11),
            }),
            AnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 13),
            }),
        ],
        vec![
            AnnotationData::Hanging(HangingAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 4),
            }),
            AnnotationData::Hanging(HangingAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 8),
            }),
        ],
        vec![
            AnnotationData::Hanging(HangingAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 4),
            }),
            AnnotationData::Label(LabelAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 8),
                label: String::from("something else"),
            }),
        ],
        vec![
            AnnotationData::Label(LabelAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 4),
                label: String::from("something"),
            }),
        ],
    ]);
}
