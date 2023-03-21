use std::collections::BTreeMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use termcolor::WriteColor;
use crate::diagnostic::{Annotation, AnnotationStyle, Diagnostic};
use crate::file::{Error, Files};
use crate::render::color::ColorConfig;
use crate::render::data::AnnotationData;

pub mod color;

mod data;
mod calculate;

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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    where FileId: Copy + Debug + Eq + Ord {
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
            let last_annotated_line_index = self.files.line_index(file, last_annotated_line_byte_offset)?;
            let last_printed_line_index = last_annotated_line_index + self.config.surrounding_lines;
            let last_printed_line_number = self.files.line_number(file, last_printed_line_index)?;

            // eprintln!("[debug] Last printed line: {}", last_printed_line_number);
            self.line_digits = last_printed_line_number.ilog10() + 1;

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
        annotations.sort_by(|a, b| a.range.start.cmp(&b.range.start));

        {
            let mut max_nested_blocks = 0;
            let mut current_nested_blocks: Vec<usize> = Vec::new();

            for annotation in annotations.iter() {
                let start_line_index = self.files.line_index(file, annotation.range.start)?;
                let end_line_index = self.files.line_index(file, annotation.range.end)?;

                if start_line_index == end_line_index {
                    continue;
                }

                current_nested_blocks.retain(|&a_end| a_end > start_line_index);
                current_nested_blocks.push(end_line_index);
                max_nested_blocks = max_nested_blocks.max(current_nested_blocks.len());
            }

            self.max_nested_blocks = max_nested_blocks;
        }

        self.render_lines_with_annotations(diagnostic, file, annotations)?;
        Ok(())
    }

    fn render_lines_with_annotations(&mut self, diagnostic: &Diagnostic<FileId>, file: FileId, annotations: Vec<Annotation<FileId>>) -> Result {
        let mut already_printed_end_index = 0;
        let mut annotations_on_line_indices = Vec::new();
        let mut continuing_annotations_indices = Vec::new();
        let mut current_line_index = 0;
        let mut last_line_index = None;
        let mut first_iteration = true;

        let last_line_index_in_file = self.files.line_index(file, self.files.source(file)?.len() - 1)?;

        loop {
            current_line_index = if first_iteration {
                current_line_index
            } else {
                current_line_index + 1
            };

            if current_line_index > last_line_index_in_file {
                break;
            }

            for (i, annotation) in annotations.iter().enumerate() {
                let start_line_index = self.files.line_index(file, annotation.range.start)?;
                let end_line_index = self.files.line_index(file, annotation.range.end)?;

                if start_line_index > current_line_index && end_line_index > current_line_index {
                    break;
                } else if end_line_index < current_line_index && start_line_index < current_line_index {
                    continue;
                }

                if continuing_annotations_indices.contains(&i) || annotations_on_line_indices.contains(&i) {
                    eprintln!("Bug in error message formatter: adding an index twice ({})!", i);
                }

                if start_line_index < current_line_index {
                    continuing_annotations_indices.push(i);
                }

                if start_line_index == current_line_index || end_line_index == current_line_index {
                    annotations_on_line_indices.push(i);
                }
            }

            if /* current_line_index != 0 && */ !annotations_on_line_indices.is_empty() {
                self.render_part_lines(diagnostic, file, current_line_index, last_line_index,
                    annotations_on_line_indices.iter().map(|i| &annotations[*i]).collect::<Vec<_>>(),
                    continuing_annotations_indices.iter().map(|i| &annotations[*i]).collect::<Vec<_>>(),
                    &mut already_printed_end_index)?;
                annotations_on_line_indices.clear();

                last_line_index = Some(current_line_index);
            }

            continuing_annotations_indices.clear();
            first_iteration = false;
        }

        if let Some(last_line) = last_line_index {
            if last_line <= self.get_last_line_index(file)? {
                self.render_post_surrounding_lines(diagnostic, file, self.get_last_line_index(file)? + 1, last_line, &[], &mut already_printed_end_index)?;
            }
        }

        Ok(())
    }

    fn render_post_surrounding_lines(&mut self, diagnostic: &Diagnostic<FileId>, file: FileId, main_line: usize, last_line: usize,
                                     continuing_annotations: &[&Annotation<FileId>],
                                     already_printed_end_line_index: &mut usize) -> Result {
        // writeln!(f, "[debug] potentially printing post surrounding lines, last line: {}, already printed to: {}", last_line, *already_printed_to)?;

        if last_line + 1 >= *already_printed_end_line_index {
            let first_print_line = (last_line + 1).max(*already_printed_end_line_index);
            let last_print_line = self.get_last_print_line(file, last_line)?.min(main_line - 1);

            // writeln!(f, "[debug] printing post surrounding lines, last line: {}, first: {}, last: {}", last_line, first_print_line, last_print_line)?;

            if last_print_line >= first_print_line {
                for line in first_print_line..=last_print_line {
                    self.render_single_source_line(diagnostic, file, line, last_line, &[], continuing_annotations)?;
                    *already_printed_end_line_index = line + 1;
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn render_part_lines(&mut self, diagnostic: &Diagnostic<FileId>, file: FileId,
                         main_line_index: usize, last_line_index: Option<usize>,
                         annotations_on_line: Vec<&Annotation<FileId>>,
                         continuing_annotations: Vec<&Annotation<FileId>>,
                         already_printed_end_line_index: &mut usize) -> Result {
        // eprintln!("[debug] Rendering part lines (main {}, last {:?}, already printed to {})", main_line_index, last_line_index.as_ref(), *already_printed_end_line_index);

        if let Some(last_line) = last_line_index {
            self.render_post_surrounding_lines(diagnostic, file, main_line_index, last_line, &continuing_annotations, already_printed_end_line_index)?;
        }

        let first_print_line_index = self.get_start_print_line(main_line_index).max(*already_printed_end_line_index);
        let last_print_line_index = main_line_index;

        // writeln!(f, "[debug] current line ({}); first = {}, last = {}", main_line, first_print_line, last_print_line)?;

        if *already_printed_end_line_index != 0 && first_print_line_index > *already_printed_end_line_index {
            self.write_source_line(diagnostic, None, "...", &continuing_annotations)?;
            writeln!(self.f)?;
        }

        for line in first_print_line_index..=last_print_line_index {
            self.render_single_source_line(diagnostic, file, line, main_line_index, &annotations_on_line, &continuing_annotations)?;
            *already_printed_end_line_index = line + 1;
        }

        Ok(())
    }

    fn render_single_source_line(&mut self, diagnostic: &Diagnostic<FileId>, file: FileId,
                                 line_index: usize, main_line_index: usize,
                                 annotations: &[&Annotation<FileId>],
                                 continuing_annotations: &[&Annotation<FileId>]) -> Result {
        self.write_source_line(diagnostic, Some((file, line_index)), " |", continuing_annotations)?;

        if line_index != main_line_index {
            return Ok(());
        }

        self.render_single_source_annotations(diagnostic, file, line_index, annotations, continuing_annotations)
    }

    fn render_single_source_annotations(&mut self, diagnostic: &Diagnostic<FileId>, file: FileId,
                                        line_index: usize,
                                        annotations: &[&Annotation<FileId>], continuing_annotations: &[&Annotation<FileId>]) -> Result {
        let data = calculate::calculate(diagnostic, &self.files, file, line_index, annotations, continuing_annotations)?;
        let mut data_stack = Vec::new();
        let mut stack_removal_indices = Vec::new();

        // eprintln!("[debug] Data:\n{:#?}", &data);

        for line_data in data.into_iter() {
            self.write_line_number(None, " |")?;

            let mut horizontal_index = 0;
            let mut last = false;

            for data in line_data.into_iter() {
                if last {
                    eprintln!("Bug in error message formatter: annotation part after label");
                }

                let to_horizontal_index = match &data {
                    AnnotationData::ContinuingMultiline(data) => data.vertical_bar_index * 2 + 1,
                    AnnotationData::ConnectingMultiline(data) => data.vertical_bar_index * 2 + 2,
                    AnnotationData::Start(data) => data.location.column_index + 2 * self.max_nested_blocks + 1,
                    AnnotationData::ConnectingSingleline(data) => data.start_column_index + 2 * self.max_nested_blocks + 1,
                    AnnotationData::End(data) => data.location.column_index + 2 * self.max_nested_blocks + 1,
                    AnnotationData::Hanging(data) => data.location.column_index + 2 * self.max_nested_blocks + 1,
                    AnnotationData::Label(data) => data.location.column_index + 2 * self.max_nested_blocks + 1,
                };

                if horizontal_index < to_horizontal_index {
                    for data in data_stack.iter().rev() {
                        self.write_annotation_data(data, Some(to_horizontal_index), &mut horizontal_index, &mut last)?;
                    }

                    for (i, data) in data_stack.iter().enumerate() {
                        let to_horizontal_index = match &data {
                            AnnotationData::ContinuingMultiline(data) => data.vertical_bar_index * 2 + 1,
                            AnnotationData::ConnectingMultiline(data) => data.end_location.column_index + 2 * self.max_nested_blocks + 1,
                            AnnotationData::Start(data) => data.location.column_index + 2 * self.max_nested_blocks + 1,
                            AnnotationData::ConnectingSingleline(data) => data.end_column_index + 2 * self.max_nested_blocks + 1,
                            AnnotationData::End(data) => data.location.column_index + 2 * self.max_nested_blocks + 1,
                            AnnotationData::Hanging(data) => data.location.column_index + 2 * self.max_nested_blocks + 1,
                            AnnotationData::Label(data) => data.location.column_index + 2 * self.max_nested_blocks + 1,
                        };

                        if to_horizontal_index < horizontal_index {
                            stack_removal_indices.push(i);
                        }
                    }

                    for (i, index) in stack_removal_indices.drain(0..stack_removal_indices.len()).enumerate() {
                        data_stack.remove(index - i);
                    }
                }

                data_stack.push(data);
            }

            for data in data_stack.iter().rev() {
                self.write_annotation_data(data, None, &mut horizontal_index, &mut last)?;
            }

            data_stack.clear();
            writeln!(self.f)?;
        }

        Ok(())
    }

    fn write_annotation_data(&mut self, data: &AnnotationData, to_horizontal_index: Option<usize>, horizontal_index: &mut usize, last: &mut bool) -> Result {
        match data {
            AnnotationData::ContinuingMultiline(data) => {
                let start = data.vertical_bar_index * 2 + 1;

                if start < *horizontal_index {
                    return Ok(());
                }

                if start > *horizontal_index {
                    write!(self.f, "{}", " ".repeat(start - *horizontal_index))?;
                    *horizontal_index = start;
                }

                self.colors.annotation(self.f, data.style, data.severity)?;
                write!(self.f, "|")?;
                self.colors.reset(self.f)?;

                *horizontal_index += 1;
            },
            AnnotationData::ConnectingMultiline(data) => {
                let start = data.vertical_bar_index * 2 + 2;
                let end = data.end_location.column_index + 2 * self.max_nested_blocks + 1;

                if end < *horizontal_index {
                    return Ok(());
                }

                if start > *horizontal_index {
                    write!(self.f, "{}", " ".repeat(start - *horizontal_index))?;
                    *horizontal_index = start;
                }

                let to_index = if let Some(to_horizontal_index) = to_horizontal_index {
                    to_horizontal_index.min(end)
                } else {
                    end
                };

                self.colors.annotation(self.f, data.style, data.severity)?;
                write!(self.f, "{}", "_".repeat(to_index - *horizontal_index))?;
                self.colors.reset(self.f)?;

                *horizontal_index = to_index;
            },
            AnnotationData::Start(data) => {
                let start = data.location.column_index + 2 * self.max_nested_blocks + 1;

                if start < *horizontal_index {
                    return Ok(());
                }

                if start > *horizontal_index {
                    write!(self.f, "{}", " ".repeat(start - *horizontal_index))?;
                    *horizontal_index = start;
                }

                self.colors.annotation(self.f, data.style, data.severity)?;
                write!(self.f, "{}", if data.style == AnnotationStyle::Primary { "^" } else { "-" })?;
                self.colors.reset(self.f)?;

                *horizontal_index += 1;
            },
            AnnotationData::ConnectingSingleline(data) => {
                let start = data.start_column_index + 2 * self.max_nested_blocks + 1;
                let end = data.end_column_index + 2 * self.max_nested_blocks + 1;

                if end < *horizontal_index {
                    return Ok(());
                }

                if start > *horizontal_index {
                    write!(self.f, "{}", " ".repeat(start - *horizontal_index))?;
                    *horizontal_index = start;
                }

                let to_index = if let Some(to_horizontal_index) = to_horizontal_index {
                    to_horizontal_index.min(end)
                } else {
                    end
                };

                self.colors.annotation(self.f, data.style, data.severity)?;
                write!(self.f, "{}", if data.as_multiline { "_" } else if data.style == AnnotationStyle::Primary { "^" } else { "-" }
                    .repeat(to_index - *horizontal_index))?;
                self.colors.reset(self.f)?;

                *horizontal_index = to_index;
            },
            AnnotationData::End(data) => {
                let start = data.location.column_index + 2 * self.max_nested_blocks + 1;

                if start < *horizontal_index {
                    return Ok(());
                }

                if start > *horizontal_index {
                    write!(self.f, "{}", " ".repeat(start - *horizontal_index))?;
                    *horizontal_index = start;
                }

                self.colors.annotation(self.f, data.style, data.severity)?;
                write!(self.f, "{}", if data.style == AnnotationStyle::Primary { "^" } else { "-" })?;
                self.colors.reset(self.f)?;

                *horizontal_index += 1;
            },
            AnnotationData::Hanging(data) => {
                let start = data.location.column_index + 2 * self.max_nested_blocks + 1;

                if start < *horizontal_index {
                    return Ok(());
                }

                if start > *horizontal_index {
                    write!(self.f, "{}", " ".repeat(start - *horizontal_index))?;
                    *horizontal_index = start;
                }

                self.colors.annotation(self.f, data.style, data.severity)?;
                write!(self.f, "|")?;
                self.colors.reset(self.f)?;

                *horizontal_index += 1;
            },
            AnnotationData::Label(data) => {
                let start = data.location.column_index + 2 * self.max_nested_blocks + 1;

                if start < *horizontal_index {
                    return Ok(());
                }

                if start > *horizontal_index {
                    write!(self.f, "{}", " ".repeat(start - *horizontal_index))?;
                    *horizontal_index = start;
                }

                self.colors.annotation(self.f, data.style, data.severity)?;
                write!(self.f, "{}", &data.label)?;
                self.colors.reset(self.f)?;

                *horizontal_index += data.label.len();
                *last = true;
            },
        }

        Ok(())
    }

    fn write_line_number(&mut self, line: Option<usize>, separator: &str) -> Result {
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

    fn write_source_line(&mut self, diagnostic: &Diagnostic<FileId>, line: Option<(FileId, usize)>, separator: &str, continuing_annotations: &[&Annotation<FileId>]) -> Result {
        let line_number = if let Some((file, line_index)) = line.as_ref() {
            Some(self.files.line_number(*file, *line_index)?)
        } else {
            None
        };

        self.write_line_number(line_number, separator)?;

        // eprintln!("[debug] writing line begin; line: {:?}, separator: {}, continuing: {}, max nested blocks: {}", line.as_ref(), separator.len(), continuing_annotations.len(), self.max_nested_blocks);

        if separator.len() < 3 && (!continuing_annotations.is_empty() || self.max_nested_blocks > 0) {
            write!(self.f, "{}", " ".repeat(3 - separator.len()))?;
        }

        for (i, annotation) in continuing_annotations.iter().enumerate() {
            self.colors.annotation(self.f, annotation.style, diagnostic.severity)?;
            write!(self.f, "|")?;
            self.colors.reset(self.f)?;

            if i < continuing_annotations.len() - 1 {
                write!(self.f, " ")?;
            }
        }

        if let Some((file, line_index)) = line {
            let source = &self.files.source(file)?[self.files.line_range(file, line_index)?];
            let is_empty = source.trim().is_empty();

            if !is_empty {
                write!(self.f, "{:>nested_blocks$}", "", nested_blocks = (2 * self.max_nested_blocks - (2 * continuing_annotations.len()).saturating_sub(1)).max(1))?;

                self.colors.source(self.f)?;

                if source.ends_with('\n') {
                    write!(self.f, "{}", source)?;
                } else {
                    writeln!(self.f, "{}", source)?;
                }

                self.colors.reset(self.f)?;
            } else {
                writeln!(self.f)?;
            }
        }

        Ok(())
    }

    fn get_start_print_line(&self, line_index: usize) -> usize {
        line_index.saturating_sub(self.config.surrounding_lines)
    }

    fn get_last_print_line(&self, file: FileId, line: usize) -> std::result::Result<usize, Error> {
        Ok((line + self.config.surrounding_lines).min(self.get_last_line_index(file)?))
    }

    fn get_last_line_index(&self, file: FileId) -> std::result::Result<usize, Error> {
        self.files.line_index(file, self.files.source(file)?.len() - 1)
    }
}

#[cfg(test)]
mod tests;
