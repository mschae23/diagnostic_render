use termcolor::{Color, ColorSpec, WriteColor};
use crate::diagnostic::{AnnotationStyle, Severity};

type Result = std::io::Result<()>;

/// Provides the terminal colors used in diagnostics.
///
/// Two default implementations are provided:
/// - [`DefaultColorConfig`], which should be similar to the colors used by `rustc`
/// - [`DisabledColorConfig`], which outputs no colors.
///
/// [`DefaultColorConfig`]: DefaultColorConfig
/// [`DisabledColorConfig`]: DisabledColorConfig
pub trait ColorConfig {
    /// Resets all style and formatting.
    fn reset(&self, f: &mut impl WriteColor) -> Result;

    /// Sets the formatting for annotations with a specific [`Severity`].
    ///
    /// [`Severity`]: Severity
    fn severity(&self, f: &mut impl WriteColor, severity: Severity) -> Result;

    /// Sets the formatting for annotations with [`Severity::Bug`].
    ///
    /// [`Severity::Bug`]: Severity::Bug
    fn bug(&self, f: &mut impl WriteColor) -> Result {
        self.severity(f, Severity::Bug)
    }

    /// Sets the formatting for annotations with [`Severity::Error`].
    ///
    /// [`Severity::Error`]: Severity::Error
    fn error(&self, f: &mut impl WriteColor) -> Result {
        self.severity(f, Severity::Error)
    }

    /// Sets the formatting for annotations with [`Severity::Warning`].
    ///
    /// [`Severity::Warning`]: Severity::Warning
    fn warning(&self, f: &mut impl WriteColor) -> Result {
        self.severity(f, Severity::Warning)
    }

    /// Sets the formatting for annotations with [`Severity::Note`].
    ///
    /// [`Severity::Note`]: Severity::Note
    fn note(&self, f: &mut impl WriteColor) -> Result {
        self.severity(f, Severity::Note)
    }

    /// Sets the formatting for annotations with [`Severity::Help`].
    ///
    /// [`Severity::Help`]: Severity::Help
    fn help(&self, f: &mut impl WriteColor) -> Result {
        self.severity(f, Severity::Help)
    }

    /// Sets the formatting for the optional error name or code.
    fn name(&self, f: &mut impl WriteColor, severity: Severity) -> Result;

    /// Sets the formatting for the main message of a diagnostic.
    fn message(&self, f: &mut impl WriteColor) -> Result;

    /// Sets the formatting for the file path, line and column numbers printed
    /// at the start of a code block.
    fn path(&self, f: &mut impl WriteColor) -> Result;

    /// Sets the formatting for the line number of a line of source code.
    fn line_number(&self, f: &mut impl WriteColor) -> Result;

    /// Sets the formatting for the separator between the line number and the line of source code.
    /// In most cases, this is either `" | "`, `"-->"`, or `"..."`.
    fn line_number_separator(&self, f: &mut impl WriteColor) -> Result;

    /// Sets the formatting for an annotation.
    /// The annotation style (primary or secondary) and the diagnostic severity are
    /// provided as context.
    ///
    /// The default configuration would redirect to [`Self::severity`] in the case of
    /// a primary annotation style, and use a specific formatting for the secondary
    /// annotation style.
    ///
    /// [`Self::severity`]: Self::severity
    fn annotation(&self, f: &mut impl WriteColor, style: AnnotationStyle, severity: Severity) -> Result;

    /// Sets the formatting for a line of source code.
    fn source(&self, f: &mut impl WriteColor) -> Result;

    fn note_severity(&self, f: &mut impl WriteColor, severity: Severity) -> Result;

    fn note_message(&self, f: &mut impl WriteColor, severity: Severity) -> Result;
}

/// The default color configuration.
/// This should be similar to the colors used in `rustc` diagnostics.
pub struct DefaultColorConfig;

impl ColorConfig for DefaultColorConfig {
    fn reset(&self, f: &mut impl WriteColor) -> Result {
        f.reset().into()
    }

    fn severity(&self, f: &mut impl WriteColor, severity: Severity) -> Result {
        f.set_color(ColorSpec::new().set_fg(Some(match severity {
            Severity::Help => Color::Green,
            Severity::Note => Color::Blue,
            Severity::Warning => Color::Yellow,
            Severity::Error | Severity::Bug => Color::Red,
        })).set_intense(matches!(severity, Severity::Note)).set_bold(matches!(severity, Severity::Note | Severity::Error | Severity::Bug)))
    }

    fn name(&self, f: &mut impl WriteColor, severity: Severity) -> Result {
        self.severity(f, severity)
    }

    fn message(&self, f: &mut impl WriteColor) -> Result {
        f.set_color(ColorSpec::new().set_bold(true))
    }

    fn path(&self, f: &mut impl WriteColor) -> Result {
        self.reset(f)
    }

    fn line_number(&self, f: &mut impl WriteColor) -> Result {
        f.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_intense(true).set_bold(true))
    }

    fn line_number_separator(&self, f: &mut impl WriteColor) -> Result {
        f.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_intense(true).set_bold(true))
    }

    fn annotation(&self, f: &mut impl WriteColor, style: AnnotationStyle, severity: Severity) -> Result {
        match style {
            AnnotationStyle::Primary => self.severity(f, severity),
            AnnotationStyle::Secondary => f.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_intense(true).set_bold(true)),
        }
    }

    fn source(&self, f: &mut impl WriteColor) -> Result {
        self.reset(f)
    }

    fn note_severity(&self, f: &mut impl WriteColor, _severity: Severity) -> Result {
        f.set_color(ColorSpec::new().set_bold(true))
    }

    fn note_message(&self, f: &mut impl WriteColor, _severity: Severity) -> Result {
        self.reset(f)
    }
}

/// A no-op color configuration.
/// Sets no formatting and outputs no formatting codes.
pub struct DisabledColorConfig;

impl ColorConfig for DisabledColorConfig {
    fn reset(&self, _f: &mut impl WriteColor) -> Result {
        Ok(())
    }

    fn severity(&self, _f: &mut impl WriteColor, _severity: Severity) -> Result {
        Ok(())
    }

    fn name(&self, _f: &mut impl WriteColor, _severity: Severity) -> Result {
        Ok(())
    }

    fn message(&self, _f: &mut impl WriteColor) -> Result {
        Ok(())
    }

    fn path(&self, _f: &mut impl WriteColor) -> Result {
        Ok(())
    }

    fn line_number(&self, _f: &mut impl WriteColor) -> Result {
        Ok(())
    }

    fn line_number_separator(&self, _f: &mut impl WriteColor) -> Result {
        Ok(())
    }

    fn annotation(&self, _f: &mut impl WriteColor, _style: AnnotationStyle, _severity: Severity) -> Result {
        Ok(())
    }

    fn source(&self, _f: &mut impl WriteColor) -> Result {
        Ok(())
    }

    fn note_severity(&self, _f: &mut impl WriteColor, _severity: Severity) -> Result {
        Ok(())
    }

    fn note_message(&self, _f: &mut impl WriteColor, _severity: Severity) -> Result {
        Ok(())
    }
}
