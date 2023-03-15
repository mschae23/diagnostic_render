use crate::diagnostic::{AnnotationStyle, Severity};
use crate::render::LineColumn;

/// Data for a continuing multi-line annotation. This is an annotation that starts
/// on a line before the currently rendered one, and ends after it.
///
/// This is drawn as a single `"|"` character to the left of the source code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContinuingMultilineAnnotationData {
    pub style: AnnotationStyle,
    pub severity: Severity,
    /// the index of this continuing vertical bar
    pub vertical_bar_index: usize,
}

/// Data for a connecting multi-line annotation. This is an annotation that is
/// running from the continuing vertical bars on the left over to its
/// location in the source code on this line.
///
/// This is used for both annotations starting and ending on a line.
/// It can only occur once per line (but of course, multiple times per source line).
///
/// This is drawn as underscores from the vertical bars to `end_location` (exclusive).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectingMultilineAnnotationData {
    pub style: AnnotationStyle,
    pub severity: Severity,
    pub end_location: LineColumn,
    /// the index of the continuing vertical bar on the left
    /// this annotation connects with
    pub vertical_bar_index: usize,
}

/// Data for a starting annotation. That is an annotation,
/// either single-line or multi-line, which starts on this line.
///
/// This is drawn as a single boundary character at `location`.
/// This can occur multiple times per line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartAnnotationLineData {
    pub style: AnnotationStyle,
    pub severity: Severity,
    pub location: LineColumn,
}

/// Data for a connecting single-line annotation. This is an annotation that is
/// entirely on a single line. This data represents the underline showing where
/// that annotation starts and ends.
///
/// This is drawn as underline characters (or underscores if `as_multiline` is `true`)
/// running from `start_column_index` (inclusive) to `end_column_index` (exclusive).
/// This can occur multiple times per line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectingSinglelineAnnotationData {
    pub style: AnnotationStyle, pub as_multiline: bool,
    pub severity: Severity,
    pub line_index: usize,
    pub start_column_index: usize, pub end_column_index: usize,
}

/// Data for an ending annotation. That is an annotation,
/// either single-line or multi-line, which ends on this line.
///
/// This is drawn as a single boundary character at `location`.
/// This can occur multiple times per line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndAnnotationLineData {
    pub style: AnnotationStyle,
    pub severity: Severity,
    pub location: LineColumn,
}

/// Data for a hanging label. This is for annotations where their
/// label would intersect with other annotations after them,
/// so they are displayed below their [`StartAnnotationLineData`].
///
/// This is drawn as a single `"|"` character at `location`.
/// This can occur multiple times per line.
///
/// [`StartAnnotationLineData`]: StartAnnotationLineData
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HangingAnnotationLineData {
    pub style: AnnotationStyle,
    pub severity: Severity,
    pub location: LineColumn,
}

/// Data for a label.
///
/// When after an [`EndAnnotationLineData`], `location` is ignored, as
/// it is the end of the line anyway. Otherwise, it is a hanging label,
/// which uses `location` for the column to print it at.
///
/// This is drawn as a label, of course, so it will simply print `label`
/// at the end of the line or at `location`.
/// This can only occur once per line.
///
/// [`EndAnnotationLineData`]: EndAnnotationLineData
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelAnnotationLineData {
    pub style: AnnotationStyle,
    pub severity: Severity,
    pub location: LineColumn,
    pub label: String,
}

/// An enum with variants for [`StartAnnotationLineData`] and
/// [`EndAnnotationLineData`], respectively.
///
/// [`StartAnnotationLineData`]: StartAnnotationLineData
/// [`EndAnnotationLineData`]: EndAnnotationLineData
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartEndAnnotationData {
    Start(StartAnnotationLineData),
    End(EndAnnotationLineData),
    Both(StartAnnotationLineData, EndAnnotationLineData),
}

/// An enum for the different types of annotation data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnnotationData {
    ContinuingMultiline(ContinuingMultilineAnnotationData),
    ConnectingMultiline(ConnectingMultilineAnnotationData),
    ConnectingSingleline(ConnectingSinglelineAnnotationData),
    End(EndAnnotationLineData),
    Hanging(HangingAnnotationLineData),
    Label(LabelAnnotationLineData),
}
