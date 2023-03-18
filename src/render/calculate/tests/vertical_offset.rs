use super::*;

mod singleline {
    use super::*;

    #[test]
    fn test_1() {
        let _file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source();\n");
        let _diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

        let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 3..12)
            .with_label("expected type annotation here");
        let annotation2 = Annotation::new(AnnotationStyle::Secondary, (), 28..31)
            .with_label("due to this");

        // 1 | let main = 23;
        //   |    ^^^^^^^^^^ expected type annotation here
        // 2 | something += 3.0;
        //   |              --- due to this

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
    fn test_2() {
        let _file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source();\n");
        let _diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

        let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 11..13)
            .with_label("number");
        let annotation2 = Annotation::new(AnnotationStyle::Secondary, (), 4..8)
            .with_label("identifier");

        // 1 | let main = 23;
        //   |     ----   ^^ number
        //   |     |
        //   |     identifier

        let starts_ends = vec![
            (&annotation2, StartEndAnnotationData::Both(StartAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 4),
            }, EndAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 8),
            })),
            (&annotation1, StartEndAnnotationData::Both(StartAnnotationLineData {
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

    #[test]
    fn test_overlapping_1() {
        let _file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source();\n");
        let _diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

        let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 4..13)
            .with_label("something");
        let annotation2 = Annotation::new(AnnotationStyle::Secondary, (), 8..11)
            .with_label("something else");

        // 1 | let main = 23;
        //   |     ^^^^---^^
        //   |     |   |
        //   |     |   something else
        //   |     something

        let starts_ends = vec![
            (&annotation1, StartEndAnnotationData::Both(StartAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 4),
            }, EndAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 13),
            })),
            (&annotation2, StartEndAnnotationData::Both(StartAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 8),
            }, EndAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 11),
            })),
        ];

        assert_eq!(calculate_vertical_offsets(&starts_ends).unwrap(), vec![2, 1]);
    }
}

mod ending {
    use super::*;

    #[test]
    fn test_1() {
        let _file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source();\n");
        let _diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

        let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 0..19)
            .with_label("something");

        // 2 | | | something += 3.0;
        //   | | |_____^ // vertical offset 0

        let starts_ends = vec![
            (&annotation1, StartEndAnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(1, 4),
            })),
        ];

        assert_eq!(calculate_vertical_offsets(&starts_ends).unwrap(), vec![0]);
    }

    #[test]
    fn test_2() {
        let _file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source();\n");
        let _diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

        let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 0..19)
            .with_label("something");
        let annotation2 = Annotation::new(AnnotationStyle::Secondary, (), 4..28)
            .with_label("something else");

        // 2 | | | something += 3.0;
        //   | | |_____^        ^ // vertical offset 0
        //   | |________________| // vertical offset 1

        let starts_ends = vec![
            (&annotation1, StartEndAnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(1, 4),
            })),
            (&annotation2, StartEndAnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(1, 13),
            })),
        ];

        assert_eq!(calculate_vertical_offsets(&starts_ends).unwrap(), vec![1, 0]);
    }

    #[test]
    fn test_overlapping_1() {
        let _file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source();\n");
        let _diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

        let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 0..28)
            .with_label("something");
        let annotation2 = Annotation::new(AnnotationStyle::Secondary, (), 4..19)
            .with_label("something else");

        // 1 |     let main = 23; // Vertical offsets for annotations on this line are not tested by this test
        //   |  ___^   ^
        //   | |  _____|
        // 2 | | | something += 3.0;
        //   | | |_____^        ^ // vertical offset 0
        //   | |________________| // vertical offset 1

        let starts_ends = vec![
            (&annotation2, StartEndAnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(1, 4),
            })),
            (&annotation1, StartEndAnnotationData::End(EndAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(1, 13),
            })),
        ];

        assert_eq!(calculate_vertical_offsets(&starts_ends).unwrap(), vec![0, 1]);
    }
}

mod starting {
    use super::*;

    #[test]
    fn test_simple_1() {
        let _file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source();\n");
        let _diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

        let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 4..28)
            .with_label("something");

        // 1 |   let main = 23;
        //   |  _____^ // vertical offset 0
        // 2 | | ...

        let starts_ends = vec![
            (&annotation1, StartEndAnnotationData::Start(StartAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 4),
            })),
        ];

        assert_eq!(calculate_vertical_offsets(&starts_ends).unwrap(), vec![0]);
    }

    #[test]
    fn test_1() {
        let _file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source();\n");
        let _diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

        let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 11..28)
            .with_label("something");
        let annotation2 = Annotation::new(AnnotationStyle::Secondary, (), 4..8)
            .with_label("something else");

        // 1 |   let main = 23;
        //   |       ^^^^   ^  // vertical offset 0
        //   |  _____|______|  // vertical offset 1
        //   | |     |         // vertical offset 2
        //   | |     something // vertical offset 3
        // 2 | | ...

        let starts_ends = vec![
            (&annotation2, StartEndAnnotationData::Both(StartAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 4),
            }, EndAnnotationLineData {
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(0, 8),
            })),
            (&annotation1, StartEndAnnotationData::Start(StartAnnotationLineData {
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 11),
            })),
        ];

        assert_eq!(calculate_vertical_offsets(&starts_ends).unwrap(), vec![2, 1]);
    }

    #[test]
    fn test_with_ending_1() {
        let _file = SimpleFile::new("test_file.test", "let main = 23;\nsomething += 3.0;\nprint(example_source();\n");
        let _diagnostic: Diagnostic<()> = Diagnostic::new(Severity::Error);

        let annotation1 = Annotation::new(AnnotationStyle::Primary, (), 28..38)
            .with_label("something"); // the one starting on line 2
        let annotation2 = Annotation::new(AnnotationStyle::Secondary, (), 11..24)
            .with_label("something else"); // the one starting on line 1, and ending on line 2

        // 1 |   let main = 23; // This line is not tested by this test
        //   |  ____________^
        // 2 | | something += 3.0;
        //   | |_________^    ^         // vertical offset 0
        //   |  _________|____|         // vertical offset 1
        //   | |         |              // vertical offset 2
        //   | |         something else // vertical offset 3 (not done in calculate vertical offsets, can't be tested here)
        // 3 | | ...

        let starts_ends = vec![
            (&annotation2, StartEndAnnotationData::End(EndAnnotationLineData {
                // the one ending on line 2
                style: AnnotationStyle::Secondary,
                severity: Severity::Error,
                location: LineColumn::new(1, 9),
            })),
            (&annotation1, StartEndAnnotationData::Start(StartAnnotationLineData {
                // the one starting on line 2
                style: AnnotationStyle::Primary,
                severity: Severity::Error,
                location: LineColumn::new(0, 11),
            })),
        ];

        assert_eq!(calculate_vertical_offsets(&starts_ends).unwrap(), vec![0, 1]);
    }
}
