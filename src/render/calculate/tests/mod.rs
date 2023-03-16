use crate::diagnostic::{AnnotationStyle, Severity};
use super::*;
use crate::file::SimpleFile;

#[test]
fn test_calculate_1() {
    let file = SimpleFile::new("test_file.test", "test file contents");
    let diagnostic = Diagnostic::new(Severity::Error);
    let annotation = Annotation::new(AnnotationStyle::Primary, (), 5..9)
        .with_label("test label");

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
        ],
    ]);
}

mod vertical_offset {
    use super::*;

    #[test]
    fn test_calculate_vertical_offset_1() {
        let _file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source();\n");
        let _diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

        let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 3..12)
            .with_label("expected type annotation here");
        let annotation2 = Annotation::new(AnnotationStyle::Secondary, (), 27..30)
            .with_label("due to this");

        let starts_ends_1 = vec![
            (&annotation1, StartEndAnnotationData::Both(StartAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 3),
            }, EndAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 12),
            })),
        ];
        let starts_ends_2 = vec![
            (&annotation2, StartEndAnnotationData::Both(StartAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(1, 13),
            }, EndAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(1, 16),
            })),
        ];

        assert_eq!(calculate_vertical_offsets(&starts_ends_1).unwrap(), vec![0]);
        assert_eq!(calculate_vertical_offsets(&starts_ends_2).unwrap(), vec![0]);
    }

    #[test]
    fn test_calculate_vertical_offset_2() {
        let _file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source();\n");
        let _diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

        let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 11..13)
            .with_label("number");
        let annotation2 = Annotation::new(AnnotationStyle::Secondary, (), 5..8)
            .with_label("identifier");

        let starts_ends = vec![
            (&annotation1, StartEndAnnotationData::Both(StartAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 5),
            }, EndAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 8),
            })),
            (&annotation2, StartEndAnnotationData::Both(StartAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 11),
            }, EndAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 13),
            })),
        ];

        assert_eq!(calculate_vertical_offsets(&starts_ends).unwrap(), vec![1, 0]);
    }
}