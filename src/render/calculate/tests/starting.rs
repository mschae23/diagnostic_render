use pretty_assertions::{assert_eq, assert_ne};
use super::*;

#[test]
fn test_simple_1() {
    let file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source);\n");
    let diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

    let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 4..28)
        .with_label("something");

    // 1 |   let main = 23;
    //   |  _____^                    // vertical offset 0
    // 2 | | something += 3.0;
    //   | |______________^ something // vertical offset 0

    // Line 1
    assert_eq!(calculate(&diagnostic, &file, (), 0, &[&annotation1], &[&annotation1]).unwrap(), vec![
        vec![
            AnnotationData::ConnectingMultiline(ConnectingMultilineAnnotationData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                end_location: LineColumn::new(0, 4),
                vertical_bar_index: 0,
            }),
            AnnotationData::Start(StartAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 4),
            }),
        ],
    ]);
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
                end_location: LineColumn::new(1, 12),
                vertical_bar_index: 0,
            }),
            AnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(1, 12),
            }),
            AnnotationData::Label(LabelAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(1, 14),
                label: String::from("something"),
            }),
        ],
    ]);
}

#[test]
fn test_1() {
    let file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source);\n");
    let diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

    let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 11..28)
        .with_label("something");
    let annotation2 = Annotation::new(AnnotationStyle::Secondary, (), 4..8)
        .with_label("something else");

    // 1 |   let main = 23;
    //   |       ----   ^             // vertical offset 0
    //   |  _____|______|             // vertical offset 1
    //   | |     |                    // vertical offset 2
    //   | |     something else       // vertical offset 3
    // 2 | | something += 3.0;
    //   | |______________^ something // vertical offset 0

    // Line 1
    assert_eq!(calculate(&diagnostic, &file, (), 0, &[&annotation2, &annotation1], &[&annotation1]).unwrap(), vec![
        vec![
            AnnotationData::Start(StartAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 4),
            }),
            AnnotationData::ConnectingSingleline(ConnectingSinglelineAnnotationData {
                style: AnnotationStyle::Secondary,
                as_multiline: false,
                severity: Severity::Error,
                line_index: 0,
                start_column_index: 4,
                end_column_index: 7,
            }),
            AnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 7),
            }),
            AnnotationData::Start(StartAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 11),
            }),
        ],
        vec![
            AnnotationData::ConnectingMultiline(ConnectingMultilineAnnotationData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                end_location: LineColumn::new(0, 11),
                vertical_bar_index: 0,
            }),
            AnnotationData::Hanging(HangingAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 4),
            }),
            AnnotationData::Hanging(HangingAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 11),
            }),
        ],
        vec![
            AnnotationData::ContinuingMultiline(ContinuingMultilineAnnotationData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                vertical_bar_index: 0,
            }),
            AnnotationData::Hanging(HangingAnnotationLineData {
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
            AnnotationData::Label(LabelAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 4),
                label: String::from("something else"),
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
                end_location: LineColumn::new(1, 12),
                vertical_bar_index: 0,
            }),
            AnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(1, 12),
            }),
            AnnotationData::Label(LabelAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(1, 14),
                label: String::from("something"),
            }),
        ],
    ]);
}

#[test]
fn test_with_ending_1() {
    let file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source);\n");
    let diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

    let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 28..38)
        .with_label("something"); // the one starting on line 2
    let annotation2 = Annotation::new(AnnotationStyle::Secondary, (), 11..24)
        .with_label("something else"); // the one starting on line 1, and ending on line 2

    // 1 |   let main = 23;
    //   |  ____________^           // vertical offset 0
    // 2 | | something += 3.0;
    //   | |_________^    ^         // vertical offset 0
    //   |  _________|____|         // vertical offset 1
    //   | |         |              // vertical offset 2
    //   | |         something else // vertical offset 3
    // 3 | | print(example_source);
    //   | |_____^ something        // vertical offset 0

    // Line 1
    assert_eq!(calculate(&diagnostic, &file, (), 0, &[&annotation2], &[&annotation2]).unwrap(), vec![
        vec![
            AnnotationData::ConnectingMultiline(ConnectingMultilineAnnotationData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                end_location: LineColumn::new(0, 11),
                vertical_bar_index: 0,
            }),
            AnnotationData::Start(StartAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 11),
            }),
        ],
    ]);
    // Line 2
    assert_eq!(calculate(&diagnostic, &file, (), 1, &[&annotation1, &annotation2], &[&annotation2, &annotation1]).unwrap(), vec![
        vec![
            AnnotationData::ContinuingMultiline(ContinuingMultilineAnnotationData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                vertical_bar_index: 0,
            }),
            AnnotationData::ConnectingMultiline(ConnectingMultilineAnnotationData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                end_location: LineColumn::new(1, 8),
                vertical_bar_index: 0,
            }),
            AnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(1, 8),
            }),
            AnnotationData::Start(StartAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(1, 13),
            }),
        ],
        vec![
            AnnotationData::ConnectingMultiline(ConnectingMultilineAnnotationData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                end_location: LineColumn::new(1, 13),
                vertical_bar_index: 0,
            }),
            AnnotationData::Hanging(HangingAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(1, 8),
            }),
            AnnotationData::Hanging(HangingAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(1, 13),
            }),
        ],
        vec![
            AnnotationData::ContinuingMultiline(ContinuingMultilineAnnotationData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                vertical_bar_index: 0,
            }),
            AnnotationData::Hanging(HangingAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(1, 8),
            }),
        ],
        vec![
            AnnotationData::ContinuingMultiline(ContinuingMultilineAnnotationData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                vertical_bar_index: 0,
            }),
            AnnotationData::Label(LabelAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(1, 8),
                label: String::from("something else"),
            }),
        ],
    ]);
}
