use crate::diagnostic::{AnnotationStyle, Severity};
use super::*;
use crate::file::SimpleFile;

mod singleline {
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
}

// TODO more tests, see examples in the comments of calculate()

mod vertical_offset;
