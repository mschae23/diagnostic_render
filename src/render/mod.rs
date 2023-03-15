use std::collections::BTreeMap;
use std::marker::PhantomData;
use termcolor::WriteColor;
use crate::diagnostic::{Annotation, AnnotationStyle, Diagnostic};
use crate::file::{Error, Files};
use crate::render::color::ColorConfig;

pub mod color;

mod data;

/// Result type for methods writing to a [`WriteColor`].
///
/// [`WriteColor`]: WriteColor
type Result = std::result::Result<(), Error>;

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

/// Contains some configuration parameters for [`DiagnosticRenderer`].
///
/// [`DiagnosticRenderer`]: DiagnosticRenderer
#[derive(Clone, Debug)]
pub struct RenderConfig {
    /// How many lines of source code to include around annotated lines for context.
    pub surrounding_lines: usize,
}

/// An ASCII renderer for diagnostics.
#[derive(Debug)]
pub struct DiagnosticRenderer<'w, W, C, FileId, F> {
    f: &'w mut W, colors: C, files: F, config: RenderConfig,
    max_nested_blocks: usize, line_digits: u32,
    _phantom_data: PhantomData<FileId>,
}

impl<'w, W, C, FileId, F> DiagnosticRenderer<'w, W, C, FileId, F> {
    /// Creates a new diagnostics renderer.
    pub fn new(f: &'w mut W, colors: C, files: F, config: RenderConfig) -> Self {
        DiagnosticRenderer {
            f, colors, files, config,
            max_nested_blocks: 0, line_digits: 0,
            _phantom_data: PhantomData,
        }
    }
}

impl<'w, W: WriteColor, C: ColorConfig, FileId, F: Files<FileId=FileId>> DiagnosticRenderer<'w, W, C, FileId, F>
    where FileId: Copy + Eq + Ord {
    /// Renders the given diagnostics.
    pub fn render(&mut self, diagnostics: Vec<Diagnostic<F::FileId>>) -> Result {
        if diagnostics.is_empty() {
            return Ok(());
        }

        self.render_impl(diagnostics)
    }

    fn render_impl(&mut self, diagnostics: Vec<Diagnostic<F::FileId>>) -> Result {
        let diagnostics_len = diagnostics.len();

        for (i, diagnostic) in diagnostics.into_iter().enumerate() {
            self.render_diagnostic(diagnostic)?;

            if i < diagnostics_len - 1 {
                writeln!(self.f)?;
            }
        }

        Ok(())
    }

    fn render_diagnostic(&mut self, mut diagnostic: Diagnostic<FileId>) -> Result {
        self.render_diagnostic_header(&diagnostic)?;

        let suppressed_count = diagnostic.suppressed_count;

        if !diagnostic.annotations.is_empty() {
            let (file, last_annotated_line_byte_offset) = diagnostic.annotations.iter()
                .map(|a| (a.file_id, a.range.end)).max_by(|(_, a), (_, b)| a.cmp(b))
                .expect("No annotations in diagnostic despite previous check");
            let last_annotated_line = self.files.line_index(file, last_annotated_line_byte_offset)?;

            let last_printed_line = last_annotated_line + self.config.surrounding_lines;

            self.line_digits = last_printed_line.ilog10() + 1;

            let annotations = diagnostic.annotations.drain(0..diagnostic.annotations.len())
                .fold(BTreeMap::<F::FileId, Vec<Annotation<F::FileId>>>::new(), |mut acc, a| {
                    acc.entry(a.file_id).or_default().push(a);
                    acc
                });

            for (file, annotations) in annotations.into_iter() {
                self.render_diagnostic_file(&diagnostic, file, annotations)?;
            }
        }

        if suppressed_count > 0 {
            writeln!(self.f, "... and {} more", suppressed_count)?;
        }

        self.max_nested_blocks = 0;
        self.line_digits = 0;

        Ok(())
    }

    fn render_diagnostic_header(&mut self, diagnostic: &Diagnostic<FileId>) -> Result {
        self.colors.severity(self.f, diagnostic.severity)?;
        write!(self.f, "{}", diagnostic.severity)?;
        // self.colors.reset(f)?;

        if let Some(name) = diagnostic.name.as_ref() {
            write!(self.f, "[")?;
            self.colors.name(self.f, diagnostic.severity)?;
            write!(self.f, "{}", name)?;
            self.colors.severity(self.f, diagnostic.severity)?;
            write!(self.f, "]")?;
        }

        if !diagnostic.message.is_empty() {
            self.colors.message(self.f)?;
            writeln!(self.f, ": {}", &diagnostic.message)?;
        }

        self.colors.reset(self.f)?;

        if diagnostic.message.is_empty() {
            writeln!(self.f)?;
        }

        Ok(())
    }

    fn render_diagnostic_file(&mut self, diagnostic: &Diagnostic<F::FileId>, file: FileId, mut annotations: Vec<Annotation<FileId>>) -> Result {
        let location = annotations.iter()
            .filter(|a| a.style == AnnotationStyle::Primary)
            .map(|a| (a.file_id, a.range.start))
            .next();

        self.write_line_number(None, "-->")?;
        write!(self.f, " ")?;
        self.colors.path(self.f)?;
        write!(self.f, "{}", self.files.name(file)?)?;

        if let Some((file, a)) = location {
            let location = self.files.location(file, a)?;
            writeln!(self.f, ":{}:{}", location.line_number, location.column_number)?;
        } else {
            writeln!(self.f)?;
        }

        self.colors.reset(self.f)?;

        // Sort by start byte index
        annotations.sort_unstable_by(|a, b| a.range.start.cmp(&b.range.start));
        self.render_lines_with_annotations(diagnostic, file, annotations)?;

        Ok(())
    }

    fn render_lines_with_annotations(&mut self, diagnostic: &Diagnostic<FileId>, file: FileId, annotations: Vec<Annotation<FileId>>) -> Result {
        let mut already_printed_to = 0;
        let mut annotations_on_line_indices = Vec::new();
        let mut continuing_annotations_indices = Vec::new();
        let mut current_line_index = 0;
        let mut last_line_index = None;
        let mut min_index = 0;

        let last_line_index_in_file = self.files.line_index(file, self.files.source(file)?.len() - 1)?;

        loop {
            current_line_index = match annotations.iter().skip(min_index).next() {
                None => break,
                Some(annotation) => (current_line_index + 1).max(self.files.line_index(file, annotation.range.start)?),
            };

            if current_line_index as usize > last_line_index_in_file {
                break;
            }

            for (i, annotation) in annotations.iter().enumerate().skip(min_index) {
                let start_line_index = self.files.line_index(file, annotation.range.start)?;
                let end_line_index = self.files.line_index(file, annotation.range.end)?;

                if start_line_index > current_line_index && end_line_index > current_line_index {
                    break;
                } else if end_line_index < current_line_index && start_line_index < current_line_index {
                    min_index = i + 1;
                    continue;
                }

                if start_line_index == current_line_index || end_line_index== current_line_index {
                    annotations_on_line_indices.push(i);
                } else if start_line_index < current_line_index && end_line_index >= current_line_index {
                    continuing_annotations_indices.push(i);
                }
            }

            if current_line_index != 0 && !annotations_on_line_indices.is_empty() {
                // Different way to render things is used here, not sure if this is needed
                // self.fix_connecting_annotations(current_line_index, &mut annotations, &annotations_on_line_indices);

                self.render_part_lines(diagnostic, file, current_line_index, last_line_index,
                    annotations_on_line_indices.iter().map(|i| &annotations[*i]).collect::<Vec<_>>(),
                    continuing_annotations_indices.iter().map(|i| &annotations[*i]).collect::<Vec<_>>(),
                    &mut already_printed_to)?;
                annotations_on_line_indices.clear();
                continuing_annotations_indices.clear();

                last_line_index = Some(current_line_index);
            }
        }

        if let Some(last_line) = last_line_index {
            if (last_line as usize) <= self.get_last_line_index(file)? {
                self.render_post_surrounding_lines(diagnostic, file, self.get_last_line_index(file)? + 1, last_line, &[], &mut already_printed_to)?;
            }
        }

        Ok(())
    }

    fn render_post_surrounding_lines(&mut self, diagnostic: &Diagnostic<FileId>, file: FileId, main_line: usize, last_line: usize,
                                    continuing_annotations: &[&Annotation<FileId>],
                                    already_printed_to_line_index: &mut usize) -> Result {
        // writeln!(f, "[debug] potentially printing post surrounding lines, last line: {}, already printed to: {}", last_line, *already_printed_to)?;

        if last_line >= *already_printed_to_line_index {
            let first_print_line = (last_line + 1).max(*already_printed_to_line_index + 1);
            let last_print_line = self.get_last_print_line(file, last_line)?.min(main_line - 1);

            // writeln!(f, "[debug] printing post surrounding lines, last line: {}, first: {}, last: {}", last_line, first_print_line, last_print_line)?;

            if last_print_line >= first_print_line {
                for line in first_print_line..=last_print_line {
                    self.render_single_source_line(diagnostic, file, line, last_line, &[], continuing_annotations)?;
                    *already_printed_to_line_index = line;
                }
            }
        }

        Ok(())
    }

    fn render_part_lines(&mut self, diagnostic: &Diagnostic<FileId>, file: FileId,
                         main_line_index: usize, last_line_index: Option<usize>,
                        annotations_on_line: Vec<&Annotation<FileId>>,
                        continuing_annotations: Vec<&Annotation<FileId>>,
                        already_printed_to_line_index: &mut usize) -> Result {
        if let Some(last_line) = last_line_index {
            self.render_post_surrounding_lines(diagnostic, file, main_line_index, last_line, &continuing_annotations, already_printed_to_line_index)?;
        }

        let first_print_line_index = self.get_start_print_line(main_line_index).max(*already_printed_to_line_index + 1);
        let last_print_line_index = main_line_index;

        // writeln!(f, "[debug] current line ({}); first = {}, last = {}", main_line, first_print_line, last_print_line)?;

        if first_print_line_index > *already_printed_to_line_index + 1 {
            self.write_line_number(None, "...")?;
            writeln!(self.f)?;
        }

        for line in first_print_line_index..=last_print_line_index {
            self.render_single_source_line(diagnostic, file, line, main_line_index, &annotations_on_line, &continuing_annotations)?;
            *already_printed_to_line_index = line;
        }

        Ok(())
    }

    fn render_single_source_line(&self, diagnostic: &Diagnostic<FileId>, file: FileId,
                                 line_index: usize, main_line_index: usize,
                                 annotations: &[&Annotation<FileId>],
                                 continuing_annotations: &[&Annotation<FileId>]) -> Result {
        Ok(())
    }

    fn write_line_number(&mut self, line: Option<u32>, separator: &str) -> Result {
        if let Some(line) = line {
            self.colors.line_number(self.f)?;
            write!(self.f, "{:>fill$}", line, fill = self.line_digits as usize)?;
        } else {
            write!(self.f, "{:>fill$}", "", fill = self.line_digits as usize)?;
        }

        self.colors.line_number_separator(self.f)?;
        write!(self.f, "{}", separator)?;
        self.colors.reset(self.f)?;
        Ok(())
    }

    fn get_start_print_line(&self, line_index: usize) -> usize {
        if self.config.surrounding_lines > line_index {
            0
        } else {
            line_index - self.config.surrounding_lines
        }
    }

    fn get_last_print_line(&self, file: FileId, line: usize) -> std::result::Result<usize, Error> {
        Ok((line + self.config.surrounding_lines).min(self.get_last_line_index(file)?))
    }

    fn get_last_line_index(&self, file: FileId) -> std::result::Result<usize, Error> {
        Ok(self.files.line_index(file, self.files.source(file)?.len() - 1)?)
    }
}

#[cfg(test)]
mod tests;
