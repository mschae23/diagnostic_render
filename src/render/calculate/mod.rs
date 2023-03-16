//! Contains the code for calculating the sequence of [`AnnotationData`]
//! for each line of source code.
//!
//! [`AnnotationData`]: AnnotationData

use std::fmt::Debug;
use crate::diagnostic::{Annotation, Diagnostic};
use crate::file::{Error, Files};
use crate::render::data::{AnnotationData, ConnectingSinglelineAnnotationData, ContinuingMultilineAnnotationData, EndAnnotationLineData, StartAnnotationLineData, StartEndAnnotationData};
use crate::render::LineColumn;

pub fn calculate<FileId: Copy + Debug>(diagnostic: &Diagnostic<FileId>, files: &impl Files<FileId=FileId>, file: FileId,
                                       line_index: usize,
                                       annotations: &[&Annotation<FileId>], continuing_annotations: &[&Annotation<FileId>]) -> Result<Vec<Vec<AnnotationData>>, Error> {
    // Create a list of the start and end points of annotations on the source line.
    // Every element is a tuple of a reference to the annotation, and its start / end data
    //
    // There are three variants of start / end data:
    // - Start: an annotation that starts on this line, and ends another one later
    // - End:   an annotation that started before this line, and ends here
    // - Both:  an annotation that both starts and ends on this line (singleline)
    //
    // Every annotation falls in one of these categories, because `annotations` only includes
    // such annotations in the first place.
    let mut starts_ends = annotations.iter()
        .fold(Result::<_, Error>::Ok(Vec::new()), |acc, &a| {
            if let Ok(mut acc) = acc {
                let start = files.line_index(file, a.range.start)?;
                let end = files.line_index(file, a.range.end)?;

                // Either start or end has to match line_index
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
    // Sort the start / end data by column index (ascending).
    // For the "both" variant, the start column index is used.
    starts_ends.sort_unstable_by(|(_, a), (_, b)|
        match a {
            StartEndAnnotationData::Start(a) | StartEndAnnotationData::Both(a, _) => a.location.column_index,
            StartEndAnnotationData::End(a) => a.location.column_index,
        }.cmp(&match b {
            StartEndAnnotationData::Start(b) | StartEndAnnotationData::Both(b, _) => b.location.column_index,
            StartEndAnnotationData::End(b) => b.location.column_index,
        }));

    eprintln!("[debug] {:#?}", &starts_ends);

    // Calculate vertical offsets
    let mut vertical_offsets = calculate_vertical_offsets(&starts_ends)?;
    eprintln!("[debug] vertical offsets: {:?}", &vertical_offsets);

    // Create a sorted vector with the vertical offsets (and an index into starts_ends)
    let mut vertical_offsets_sorted = vertical_offsets.iter().enumerate()
        .map(|(i, offset)| (i, *offset)).collect::<Vec<_>>();
    vertical_offsets_sorted.sort_unstable_by(|(_, a), (_, b)| a.cmp(b)); // sort by the vertical offset
    // How many elements from the start of continuing_annotations to take
    // len() - 1 could cause an underflow (panic), and take() stops when the end of the iter is reached anyway
    let mut continuing_take_index: usize = continuing_annotations.len();

    for (i, a) in continuing_annotations.iter().enumerate().rev() {
        if files.line_index(file, a.range.start)? < line_index {
            continuing_take_index = i;
            break;
        }
    }

    // Create ContinuingMultiline data for the continuing annotations at the start.
    // Here, this is done only for the first line (which has the underlines)
    let data = continuing_annotations.iter().take(continuing_take_index)
        .fold(Vec::new(), |mut acc, a| {
            acc.push(AnnotationData::ContinuingMultiline(ContinuingMultilineAnnotationData {
                style: a.style,
                severity: diagnostic.severity,
                vertical_bar_index: acc.len(),
            }));
            acc
        });

    let data = vertical_offsets.iter().enumerate().fold(data, |mut acc, (i, _)| {
        let (annotation, start_end) = &starts_ends[i];

        match start_end {
            StartEndAnnotationData::Start(start) => {
                acc.push(AnnotationData::Start(start.clone()));
            },
            StartEndAnnotationData::End(end) => {
                acc.push(AnnotationData::End(end.clone()));
            },
            StartEndAnnotationData::Both(start, end) => {
                acc.push(AnnotationData::Start(start.clone()));
                acc.push(AnnotationData::ConnectingSingleline(ConnectingSinglelineAnnotationData {
                    style: annotation.style,
                    as_multiline: false,
                    severity: diagnostic.severity,
                    line_index,
                    // Intersects with the start boundary character, but the renderer will prefer
                    // that one over this connecting line anyway
                    start_column_index: start.location.column_index,
                    end_column_index: end.location.column_index,
                }));
                acc.push(AnnotationData::End(end.clone()));
            },
        };

        acc
    });

    let (_, data) = vertical_offsets_sorted.iter().fold((0, vec![data]), |(last_vertical_offset, mut acc), (i, offset)| {
        let i = *i;
        let vertical_offset = *offset;
        let (_annotation, _start_end) = &starts_ends[i];

        if vertical_offset > last_vertical_offset {
            acc.push(Vec::new());
        }

        // TODO

        (vertical_offset, acc)
    });

    Ok(data)
}

fn calculate_vertical_offsets<FileId: Copy + Debug>(starts_ends: &[(&Annotation<FileId>, StartEndAnnotationData)]) -> Result<Vec<u32>, Error> {
    let mut vertical_offsets = vec![0u32; starts_ends.len()];
    let mut next_vertical_offset: u32 = 0;
    let mut processed = vec![false; starts_ends.len()];

    // Process the single-line annotations (with start / end data "both")
    //
    // For this, the start / end data vector is iterated in reverse and given incrementing
    // vertical offsets.
    // This means that the rightmost annotations (by start column index) are given lower offsets
    // than ones that come before them on the line.
    //
    // Examples:
    //
    // Here, the annotations are not overlapping. You can see that they are assigned their
    // vertical offset from right to left.
    // 23 | pub fn example_function(&mut self, argument: usize) -> usize {
    //    |                         ---------  --------            ----- return type
    //    |                         |          |
    //    |                         |          a parameter
    //    |                         self parameter
    //
    // Here, there are two overlapping annotations. They are still assigned their vertical
    // offset from right to left.
    // 23 | pub fn example_function(&mut self, argument: usize) -> usize {
    //    |                        ------------^^^^^^^^^^^^^^^-
    //    |                        |           |
    //    |                        |           a parameter
    //    |                        the parameter list
    for (i, (a, start_end)) in starts_ends.iter().enumerate().rev() {
        match start_end {
            StartEndAnnotationData::Both(_, _) => {
                if a.label.is_empty() {
                    // If a single-line annotation has no label, it doesn't take vertical space

                    if i == 0 {
                        // Except if it's the rightmost one, in which case the next annotation
                        // has to start on vertical offset 1
                        next_vertical_offset += 1;
                    }

                    processed[i] = true;
                    continue;
                }

                vertical_offsets[i] = next_vertical_offset;
                next_vertical_offset += 1;
                processed[i] = true;
            },
            // Ignore multi-line annotations
            StartEndAnnotationData::Start(_) => continue,
            StartEndAnnotationData::End(_) => continue,
        }
    }

    {
        // for multi-line annotations ending on this line, stores where they started (as byte index)
        let mut start_byte_indices = vec![None; starts_ends.len()];

        // Iterate through start / end data to fill start_byte_indices
        for (i, (a, start_end)) in starts_ends.iter().enumerate() {
            match start_end {
                StartEndAnnotationData::End(_) => {
                    start_byte_indices[i] = Some((i, a.range.start));
                },
                StartEndAnnotationData::Start(_) => continue,
                StartEndAnnotationData::Both(_, _) => continue,
            }
        }

        // only keep the elements which are actually ending multi-line annotations
        // because this changes the indices, the index of the corresponding annotations
        // in the starts_ends vector was saved with the byte index in a tuple above
        start_byte_indices.retain(|a| a.is_some());
        // Unwrap all the Option values.
        // This shouldn't panic, as we have removed all None elements before.
        let mut starts = start_byte_indices.into_iter().map(|a| a.expect("`None` despite previous check")).collect::<Vec<_>>();
        // Sort by start byte index (ascending)
        starts.sort_unstable_by(|(_, a), (_, b)| a.cmp(&b));

        // Iterates through all multi-line annotations ending on this line
        // in descending start byte index order, to be able to assign lower vertical offsets
        // to the continuing vertical bars that are more on the right, to avoid intersecting lines.
        //
        // The order is important so that it doesn't look like this:
        // 23 | | | pub fn example_function(&mut self, argument: usize) -> usize {
        //    | |_|___^    ^
        //    |   |___|____|
        //    |       |    some label
        //    |       some other label
        //
        // It should look like this:
        // 23 | | | pub fn example_function(&mut self, argument: usize) -> usize {
        //    | | |   ^    ^
        //    | | |___|____|
        //    | |_____|    some label
        //    |       some other label
        //
        // So multi-line ending annotations are assigned incrementing vertical offsets the smaller
        // their start byte index is.
        //
        // Note that these can get an additional vertical offset when starting multi-line
        // annotations need to intersect with them:
        // 23 | | | pub fn example_function(&mut self, argument: usize) -> usize {
        //    | | |   ^    ^                                                     ^
        //    | | |___|____|                                                     |
        //    | |_____|    |                                                     |
        //    |  _____|____|_____________________________________________________|
        //    | |     |    |
        //    | |     |    some label
        //    | |     some other label
        // This is something that is calculated later, though.
        for (i, _) in starts.into_iter().rev() {
            vertical_offsets[i] = next_vertical_offset;
            next_vertical_offset += 1;
            processed[i] = true;
        }
    }

    // Iterate through starts_ends again, for the multi-line starting annotations.
    // Vertical offsets are assigned incrementing vertical offsets in their order
    // from left to right (which matches the above assumption that annotations with
    // earlier start byte indices have a continuing vertical bar further on the left).
    //
    // Example:
    //
    // Here, there is only a single annotation, so it should simply run over to the left
    // on the same line that is used for underlines.
    // 23 |     pub fn example_function(&mut self, argument: usize) -> usize {
    //    |  ________________________________________________________________^
    //    | |
    //
    // With a single-line annotation before it:
    // 23 |     pub fn example_function(&mut self, argument: usize) -> usize {
    //    |                                        ---------------           ^
    //    |                                        |                         |
    //    |                                        a parameter               |
    //    |  ________________________________________________________________|
    //    | |
    //
    // With an ending annotation before it:
    // 23 | |   pub fn example_function(&mut self, argument: usize) -> usize {
    //    | |_____________________________________________________^          ^
    //    |  _____________________________________________________|__________|
    //    | |                                                     |
    //    | |                                                     a parameter list
    for (i, (_, start_end)) in starts_ends.iter().enumerate() {
        match start_end {
            StartEndAnnotationData::Start(_) => {
                vertical_offsets[i] = next_vertical_offset;
                next_vertical_offset += 1;
                processed[i] = true;
            },
            StartEndAnnotationData::Both(_, _) => continue,
            StartEndAnnotationData::End(_) => continue,
        }
    }

    // Assert that all annotations have been given a vertical offset
    // (so that it is false that any annotation has not been given one)
    assert!(!processed.into_iter().any(|x| !x), "an annotation has not been given a vertical offset");
    Ok(vertical_offsets)
}

#[cfg(test)]
mod tests;
