//! Contains the code for calculating the sequence of [`AnnotationData`]
//! for each line of source code.
//!
//! [`AnnotationData`]: AnnotationData

use std::fmt::Debug;
use crate::diagnostic::{Annotation, Diagnostic};
use crate::file::{Error, Files};
use crate::render::data::{AnnotationData, ContinuingMultilineAnnotationData, EndAnnotationLineData, StartAnnotationLineData, StartEndAnnotationData};
use crate::render::LineColumn;

pub fn calculate<FileId: Copy + Debug>(diagnostic: &Diagnostic<FileId>, files: &impl Files<FileId=FileId>, file: FileId,
                 line_index: usize,
                 annotations: &[&Annotation<FileId>], continuing_annotations: &[&Annotation<FileId>]) -> Result<Vec<Vec<AnnotationData>>, Error> {
    let mut data = continuing_annotations.iter()
        .fold(Vec::new(), |mut acc, a| {
            acc.push(AnnotationData::ContinuingMultiline(ContinuingMultilineAnnotationData {
                style: a.style,
                severity: diagnostic.severity,
                vertical_bar_index: acc.len(),
            }));
            acc
        });

    let mut starts_ends = annotations.iter()
        .fold(Result::<_, Error>::Ok(Vec::new()), |acc, &a| {
            if let Ok(mut acc) = acc {
                let start = files.line_index(file, a.range.start)?;
                let end = files.line_index(file, a.range.end)?;

                // Either start or end has to match line_index, because calculate should only
                // get called with annotations on line_index in the first place
                let start_part = if start == line_index {
                    Some(StartAnnotationLineData {
                        style: a.style,
                        severity: diagnostic.severity,
                        location: LineColumn::new(line_index, a.range.start - files.line_range(file, start)?.start),
                    })
                } else { None };

                let end_part = if end == line_index {
                   Some(EndAnnotationLineData {
                        style: a.style,
                        severity: diagnostic.severity,
                        location: LineColumn::new(line_index, a.range.end - files.line_range(file, end)?.start),
                    })
                } else { None };

                match (start_part, end_part) {
                    (Some(start_part), Some(end_part)) => acc.push((a, StartEndAnnotationData::Both(start_part, end_part))),
                    (Some(start_part), _) => acc.push((a, StartEndAnnotationData::Start(start_part))),
                    (_, Some(end_part)) => acc.push((a, StartEndAnnotationData::End(end_part))),
                    _ => panic!("Annotation neither starts nor ends in this line, despite previous check"),
                };

                Ok(acc)
            } else {
                acc
            }
        })?;
    starts_ends.sort_unstable_by(|(_, a), (_, b)|
        match a {
            StartEndAnnotationData::Start(a) | StartEndAnnotationData::Both(a, _) => a.location.column_index,
            StartEndAnnotationData::End(a) => a.location.column_index,
        }.cmp(&match b {
            StartEndAnnotationData::Start(b) | StartEndAnnotationData::Both(b, _) => b.location.column_index,
            StartEndAnnotationData::End(b) => b.location.column_index,
        }));

    eprintln!("[debug] {:#?}", &starts_ends);
    let mut vertical_offsets = vec![0u32; starts_ends.len()];

    {
        let mut next_vertical_offset: u32 = 0;
        let mut starts = vec![None; starts_ends.len()];

        for (i, (a, start_end)) in starts_ends.iter().enumerate().rev() {
            match start_end {
                StartEndAnnotationData::Both(_, _) => {
                    if a.label.is_empty() {
                        continue; // If a singleline annotation has no label, it doesn't take vertical space
                    }

                    vertical_offsets[i] = next_vertical_offset;
                    next_vertical_offset += 1;
                },
                StartEndAnnotationData::Start(_) => continue,
                StartEndAnnotationData::End(_) => continue,
            }
        }

        for (i, (a, start_end)) in starts_ends.iter().enumerate() {
            match start_end {
                StartEndAnnotationData::End(_) => {
                    starts[i] = Some((i, a.range.start));
                },
                StartEndAnnotationData::Start(_) => continue,
                StartEndAnnotationData::Both(_, _) => continue,
            }
        }

        starts.retain(|a| a.is_some());
        let mut starts = starts.into_iter().map(|a| a.expect("`None` despite previous check")).collect::<Vec<_>>();
        starts.sort_unstable_by(|(_, a), (_, b)| a.cmp(&b));

        for (i, _) in starts.into_iter().rev() {
            vertical_offsets[i] = next_vertical_offset;
            next_vertical_offset += 1;
        }

        for (i, (_, start_end)) in starts_ends.iter().enumerate() {
            match start_end {
                StartEndAnnotationData::Start(_) => {
                    vertical_offsets[i] = next_vertical_offset;
                    next_vertical_offset += 1;
                },
                StartEndAnnotationData::Both(_, _) => continue,
                StartEndAnnotationData::End(_) => continue,
            }
        }
    }

    eprintln!("[debug] vertical offsets: {:?}", &vertical_offsets);
    Ok(vec![data])
}

#[cfg(test)]
mod tests {
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
            ],
        ]);
    }
}
