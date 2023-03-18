use pretty_assertions::{assert_eq, assert_ne};
use super::*;

#[test]
fn test_1() {
    let file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source();\n");
    let diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

    let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 0..19)
        .with_label("something");

    // 1 |   let main = 23;
    //   |  _^
    // 2 | | something += 3.0;
    //   | |_____^ // vertical offset 0

    // Line 1
    assert_eq!(calculate(&diagnostic, &file, (), 0, &[&annotation1], &[&annotation1]).unwrap(), vec![
        vec![
            AnnotationData::ConnectingMultiline(ConnectingMultilineAnnotationData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                end_location: LineColumn::new(0, 0),
                vertical_bar_index: 0,
            }),
            AnnotationData::Start(StartAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 0),
            }),
        ],
    ]);
    // Line 2
    assert_eq!(calculate(&diagnostic, &file, (), 1, &[&annotation1], &[&annotation1]).unwrap(), vec![
        vec![
            AnnotationData::ContinuingMultiline(ContinuingMultilineAnnotationData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                vertical_bar_index: 0,
            }),
            AnnotationData::ConnectingMultiline(ConnectingMultilineAnnotationData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                end_location: LineColumn::new(1, 4),
                vertical_bar_index: 0,
            }),
            AnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(1, 4),
            }),
            AnnotationData::Label(LabelAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(1, 5),
                label: String::from("something"),
            }),
        ],
    ]);
}

#[test]
fn test_2() {
    let file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source();\n");
    let diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

    let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 0..28)
        .with_label("something");
    let annotation2 = Annotation::new(AnnotationStyle::Secondary, (), 4..19)
        .with_label("something else");

    // 1 |     let main = 23;
    //   |  ___^   ^
    //   | |  _____|
    // 2 | | | something += 3.0;
    //   | | |_____^        ^ // vertical offset 0
    //   | |________________| // vertical offset 1

    assert_eq!(calculate(&diagnostic, &file, (), 0, &[&annotation1, &annotation2], &[&annotation1, &annotation2]).unwrap(), vec![
        vec![
            AnnotationData::ConnectingMultiline(ConnectingMultilineAnnotationData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                end_location: LineColumn::new(0, 0),
                vertical_bar_index: 0,
            }),
            AnnotationData::Start(StartAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 0),
            }),
            AnnotationData::Start(StartAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 4),
            }),
        ],
        vec![
            AnnotationData::ContinuingMultiline(ContinuingMultilineAnnotationData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                vertical_bar_index: 0,
            }),
            AnnotationData::ConnectingMultiline(ConnectingMultilineAnnotationData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                end_location: LineColumn::new(0, 4),
                vertical_bar_index: 1,
            }),
            AnnotationData::Hanging(HangingAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 4),
            }),
        ],
    ]);

    // TODO label for secondary annotation, which has to intersect with the connecting line for the primary annotation
    assert_eq!(calculate(&diagnostic, &file, (), 1, &[&annotation1, &annotation2], &[&annotation1, &annotation2]).unwrap(), vec![
        vec![
            AnnotationData::ContinuingMultiline(ContinuingMultilineAnnotationData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                vertical_bar_index: 0,
            }),
            AnnotationData::ContinuingMultiline(ContinuingMultilineAnnotationData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                vertical_bar_index: 1,
            }),
            AnnotationData::ConnectingMultiline(ConnectingMultilineAnnotationData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                end_location: LineColumn::new(1, 4),
                vertical_bar_index: 1,
            }),
            AnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(1, 4),
            }),
            AnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(1, 13),
            }),
            AnnotationData::Label(LabelAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(1, 14),
                label: String::from("something"),
            }),
        ],
        vec![
            AnnotationData::ContinuingMultiline(ContinuingMultilineAnnotationData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                vertical_bar_index: 0,
            }),
            AnnotationData::ConnectingMultiline(ConnectingMultilineAnnotationData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                end_location: LineColumn::new(1, 13),
                vertical_bar_index: 0,
            }),
            AnnotationData::Hanging(HangingAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(1, 13),
            }),
        ],
    ]);
}
