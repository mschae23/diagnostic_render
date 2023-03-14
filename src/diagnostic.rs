//! Diagnostic data structures.
//!
//! End users are encouraged to create their own implementations
//! for their specific use cases, and convert them to this crate's
//! representation when needed.

use std::fmt::{Display, Formatter};
use std::ops::Range;

/// A severity level for diagnostic messages.
///
/// These are ordered in the following way:
///
/// ```rust
/// use diagnostic_render::diagnostic::Severity;
///
/// assert!(Severity::Bug > Severity::Error);
/// assert!(Severity::Error > Severity::Warning);
/// assert!(Severity::Warning > Severity::Note);
/// assert!(Severity::Note > Severity::Help);
/// ```
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// A help message
    Help,
    /// A note
    Note,
    /// A warning
    Warning,
    /// An error
    Error,
    /// An unexpected bug
    Bug,
}

impl Display for Severity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Severity::Bug => "bug",
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
            Severity::Help => "help",
        })
    }
}

/// A style for annotations.
///
/// These are ordered in the following way:
/// ```rust
/// use diagnostic_render::diagnostic::AnnotationStyle;
///
/// assert!(AnnotationStyle::Primary < AnnotationStyle::Secondary);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnnotationStyle {
    /// Annotations that describe the primary cause of a diagnostic.
    Primary,
    /// Annotations that provide additional context for a diagnostic.
    Secondary,
}

/// An annotation describing an underlined region of code associated with a diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Annotation<FileId> {
    /// The style of the annotation.
    pub style: AnnotationStyle,
    /// The file that we are annotating.
    pub file_id: FileId,
    /// The range in bytes we are going to include in the final snippet.
    pub range: Range<usize>,
    /// An optional label to provide some additional information for the
    /// underlined code. These should not include line breaks.
    pub label: String,
}

impl<FileId> Annotation<FileId> {
    /// Create a new annotation with no label.
    pub fn new<R: Into<Range<usize>>>(style: AnnotationStyle, file_id: FileId, range: R) -> Self {
        Annotation {
            style,
            file_id,
            range: range.into(),
            label: String::new(),
        }
    }

    /// Create a new annotation with a style of [`AnnotationStyle::Primary`].
    ///
    /// [`AnnotationStyle::Primary`]: AnnotationStyle::Primary
    pub fn primary<R: Into<Range<usize>>>(file_id: FileId, range: R) -> Self {
        Self::new(AnnotationStyle::Primary, file_id, range)
    }

    /// Create a new label with a style of [`AnnotationStyle::Secondary`].
    ///
    /// [`AnnotationStyle::Secondary`]: AnnotationStyle::Secondary
    pub fn secondary<R: Into<Range<usize>>>(file_id: FileId, range: R) -> Self {
        Self::new(AnnotationStyle::Secondary, file_id, range)
    }

    /// Add a label to the annotation.
    pub fn with_label<L: ToString>(mut self, label: L) -> Self {
        self.label = label.to_string();
        self
    }
}

/// A note associated with the primary cause of a diagnostic.
/// They can be used to explain the diagnostic, or include help
/// on how to fix an issue.
///
/// They are displayed at the end of diagnostics, after the source code with
/// its annotations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    /// The severity of the note.
    ///
    /// This should usually only be [`Severity::Help`] or [`Severity::Note`].
    ///
    /// [`Severity::Help`]: Severity::Help
    /// [`Severity::Note`]: Severity::Note
    pub severity: Severity,
    /// The message of this note.
    /// This can include line breaks for improved formatting.
    /// It should not be empty.
    pub message: String,
}

impl Note {
    /// Create a new note.
    pub fn new<M: ToString>(severity: Severity, message: M) -> Self {
        Note {
            severity, message: message.to_string(),
        }
    }

    /// Create a new note with a severity of [`Severity::Note`].
    ///
    /// [`Severity::Note`]: Severity::Note
    pub fn note<M: ToString>(message: M) -> Self {
        Self::new(Severity::Note, message)
    }

    /// Create a new note with a severity of [`Severity::Help`].
    ///
    /// [`Severity::Help`]: Severity::Help
    pub fn help<M: ToString>(message: M) -> Self {
        Self::new(Severity::Help, message)
    }
}

/// Represents a diagnostic message that can provide information like errors and
/// warnings to the user.
///
/// The position of a Diagnostic is considered to be the position of the [`Annotation`]
/// that has the earliest starting position and has the highest style which appears
/// in all the annotations of the diagnostic.
///
/// [`Annotation`]: Annotation
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic<FileId> {
    /// The overall severity of the diagnostic.
    pub severity: Severity,
    /// An optional name or code that identifies this diagnostic.
    pub name: Option<String>,
    /// The main message associated with this diagnostic.
    ///
    /// These should not include line breaks, and in order support the 'short'
    /// diagnostic display style, the message should be specific enough to make
    /// sense on its own, without additional context provided by annotations and notes.
    pub message: String,
    /// Source annotations that describe the cause of the diagnostic.
    ///
    /// The order of the annotations inside the vector does not have any meaning.
    /// The annotations are always arranged in the order they appear in the source code.
    pub annotations: Vec<Annotation<FileId>>,
    /// Notes that are associated with the primary cause of the diagnostic.
    pub notes: Vec<Note>,

    // /// Additional diagnostics that can be used to show context from other files,
    // /// provide help by showing changed code, or similar. They are shown below notes.
    // pub sub_diagnostics: Vec<Diagnostic<FileId>>,

    /// The number of diagnostics following this one that are hidden due to
    /// something like panic mode in error reporting.
    pub suppressed_count: u32,
}

impl<FileId> Diagnostic<FileId> {
    /// Create a new diagnostic.
    pub fn new(severity: Severity) -> Self {
        Diagnostic {
            severity,
            name: None,
            message: String::new(),
            annotations: Vec::new(),
            notes: Vec::new(),
            suppressed_count: 0,
        }
    }

    /// Create a new diagnostic with a severity of [`Severity::Bug`].
    ///
    /// [`Severity::Bug`]: Severity::Bug
    pub fn bug() -> Self {
        Self::new(Severity::Bug)
    }

    /// Create a new diagnostic with a severity of [`Severity::Error`].
    ///
    /// [`Severity::Error`]: Severity::Error
    pub fn error() -> Self {
        Self::new(Severity::Error)
    }

    /// Create a new diagnostic with a severity of [`Severity::Warning`].
    ///
    /// [`Severity::Warning`]: Severity::Warning
    pub fn warning() -> Self {
        Self::new(Severity::Warning)
    }

    /// Create a new diagnostic with a severity of [`Severity::Note`].
    ///
    /// [`Severity::Note`]: Severity::Note
    pub fn note() -> Self {
        Self::new(Severity::Note)
    }

    /// Create a new diagnostic with a severity of [`Severity::Help`].
    ///
    /// [`Severity::Help`]: Severity::Help
    pub fn help() -> Self {
        Self::new(Severity::Help)
    }

    /// Set the name or code of the diagnostic.
    pub fn with_name<M: ToString>(mut self, name: M) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Set the message of the diagnostic.
    pub fn with_message<M: ToString>(mut self, message: M) -> Self {
        self.message = message.to_string();
        self
    }

    /// Add an annotation to the diagnostic.
    pub fn with_annotation(mut self, annotation: Annotation<FileId>) -> Self {
        self.annotations.push(annotation);
        self
    }

    /// Add some labels to the diagnostic.
    pub fn with_annotations(mut self, mut annotations: Vec<Annotation<FileId>>) -> Self {
        self.annotations.append(&mut annotations);
        self
    }

    /// Add a note to the diagnostic.
    pub fn with_note(mut self, note: Note) -> Self {
        self.notes.push(note);
        self
    }

    /// Add some notes to the diagnostic.
    pub fn with_notes(mut self, mut notes: Vec<Note>) -> Self {
        self.notes.append(&mut notes);
        self
    }

    /// Sets the number of suppressed diagnostics.
    pub fn with_suppressed_count(mut self, suppressed_count: u32) -> Self {
        self.suppressed_count = suppressed_count;
        self
    }
}
