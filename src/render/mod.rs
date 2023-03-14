use termcolor::WriteColor;
use crate::diagnostic::Diagnostic;
use crate::file::Files;
use crate::render::color::ColorConfig;

pub mod color;

mod data;

/// Represents a location in a specific source file,
/// using line and column indices.
///
/// Note that these are indices and not user-facing numbers,
/// so they are `0`-indexed.
///
/// It is not necessarily checked that this position exists
/// in the source file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineColumn {
    /// The `0`-indexed line index.
    pub line_index: usize,
    /// The `0`-indexed column index.
    pub column_index: usize,
}

impl LineColumn {
    /// Creates a new location.
    pub fn new(line_index: usize, column_index: usize) -> Self {
        LineColumn {
            line_index, column_index,
        }
    }
}

impl From<(usize, usize)> for LineColumn {
    #[inline]
    fn from((line_index, column_index): (usize, usize)) -> Self {
        Self::new(line_index, column_index)
    }
}

/// An ASCII renderer for diagnostics.
#[derive(Debug)]
pub struct DiagnosticRenderer<'a, 'w, W, C, F: Files<'a>> {
    f: &'w mut W, colors: C,
    diagnostics: Vec<Diagnostic<F::FileId>>,
}

impl<'a, 'w, W, C, F: Files<'a>> DiagnosticRenderer<'a, 'w, W, C, F> {
    /// Creates a new diagnostics renderer.
    pub fn new_with_diagnostics(f: &'w mut W, colors: C, diagnostics: Vec<Diagnostic<F::FileId>>) -> Self {
        DiagnosticRenderer {
            f, colors,
            diagnostics,
        }
    }

    /// Creates a new diagnostics renderer.
    pub fn new(f: &'w mut W, colors: C) -> Self {
        Self::new_with_diagnostics(f, colors, Vec::new())
    }

    /// Appends `diagnostic` to this renderer.
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic<F::FileId>) {
        self.diagnostics.push(diagnostic);
    }

    /// Appends `diagnostics` to this renderer, leaving `diagnostics` empty.
    pub fn add_diagnostics(&mut self, diagnostics: &mut Vec<Diagnostic<F::FileId>>) {
        self.diagnostics.append(diagnostics);
    }
}

impl<'a, 'w, W: WriteColor, C: ColorConfig, F: Files<'a>> DiagnosticRenderer<'a, 'w, W, C, F> {
    /// Renders all diagnostics.
    ///
    /// The renderer will no longer contain any diagnostics after this call.
    pub fn render(&mut self) {
        if self.diagnostics.is_empty() {
            return;
        }

        todo!()
    }
}
