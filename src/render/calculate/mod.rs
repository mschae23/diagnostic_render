//! Contains the code for calculating the sequence of [`AnnotationData`]
//! for each line of source code.
//!
//! [`AnnotationData`]: AnnotationData

use std::fmt::Debug;
use crate::diagnostic::{Annotation, Diagnostic};
use crate::file::{Error, Files};
use crate::render::data::{AnnotationData, ConnectingMultilineAnnotationData, ConnectingSinglelineAnnotationData, ContinuingMultilineAnnotationData, EndAnnotationLineData, HangingAnnotationLineData, LabelAnnotationLineData, StartAnnotationLineData, StartEndAnnotationData};
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
    let vertical_offsets = calculate_vertical_offsets(&starts_ends)?;
    eprintln!("[debug] vertical offsets: {:?}", &vertical_offsets);

    let final_data = calculate_final_data(diagnostic, files, file, line_index, &starts_ends, vertical_offsets, continuing_annotations)?;
    Ok(final_data)
}

fn calculate_vertical_offsets<FileId: Copy + Debug>(starts_ends: &[(&Annotation<FileId>, StartEndAnnotationData)]) -> Result<Vec<u32>, Error> {
    let mut vertical_offsets = vec![0u32; starts_ends.len()];
    let mut next_vertical_offset: u32 = 0;
    let mut processed = vec![false; starts_ends.len()];

    // Starting annotations need to come before single-line, but after ending annotations.
    // However, assigning vertical offsets in that order would also be incorrect, as
    // single-line annotations need to get smaller vertical offsets than ending annotations.
    //
    // Instead, we just calculate a static offset here, which is equal to the number of starting
    // annotations on this line.
    // This only gets applied if the rightmost annotation is a single-line one (in the
    // code iterating through starts_ends for "Both" start / end data).
    let static_offset_from_start: u32 = starts_ends.iter().enumerate().fold(0, |acc, (_, (_, start_end))| match start_end {
        StartEndAnnotationData::Start(_) => acc + 1,
        StartEndAnnotationData::End(_) => acc,
        StartEndAnnotationData::Both(_, _) => acc,
    });
    // Used for asserting that we aren't assigning vertical offsets to starting annotations
    // that have already been used for other ones.
    //
    // We can't just compare the vertical offset currently being assigned to "next_vertical_offset"
    // in the code where this is used, as it is both valid for it to be smaller and bigger than that.
    let mut end_offset_for_start = 0;

    // eprintln!("[debug] static_offset_from_start: {}", static_offset_from_start);

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
            StartEndAnnotationData::Both(start, _) => {
                if a.label.is_empty() {
                    // If a single-line annotation has no label, it doesn't take vertical space

                    if i == starts_ends.len() - 1 {
                        // Except if it's the rightmost one, in which case the next annotation
                        // has to start on vertical offset 1
                        next_vertical_offset += 1;
                    }

                    processed[i] = true;
                    continue;
                }

                // Special case for when there is a rightmost single-line annotation,
                // but another one ends after that one starts.
                // In this case, all vertical offsets need to be incremented by 1.
                if next_vertical_offset == 0 {
                    // Iterate through starts_ends again (same order, in reverse)
                    // The last one has to be skipped, as that is definitely this one
                    // and will make the condition always match
                    for (_j, (_, start_end_2)) in starts_ends.iter().enumerate().rev().skip(1) {
                        let end = match start_end_2 {
                            // If one of these ends after the rightmost single-line annotation,
                            // increase vertical_offset by 1 for all annotations
                            StartEndAnnotationData::Start(start) => start.location.column_index,
                            StartEndAnnotationData::End(end) => end.location.column_index,
                            StartEndAnnotationData::Both(_, end) => end.location.column_index,
                        };

                        if end >= start.location.column_index {
                            next_vertical_offset += 1;
                            break;
                        }
                    }

                    // Apply the static offset to give space for starting annotations
                    // at the beginning
                    end_offset_for_start = next_vertical_offset + static_offset_from_start;
                    next_vertical_offset += static_offset_from_start;
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
        // This is something that is calculated later, not in this function.
        for (i, _) in starts.into_iter().rev() {
            vertical_offsets[i] = next_vertical_offset;
            next_vertical_offset += 1;
            processed[i] = true;
        }
    }

    // Starting annotations use a different "next_vertical_offset" variable
    // because they need to use the space given to them by the static offset applied above.
    let mut next_start_vertical_offset = if next_vertical_offset > 0 { 1 } else { 0 };

    if next_start_vertical_offset >= next_vertical_offset {
        // If this starting annotation would've gotten the regular
        // next vertical offset anyway, adjust end offset to ensure
        // we don't panic due to the assertion

        // eprintln!("[debug] resetting end offset for start");
        end_offset_for_start = u32::MAX;
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
    //    |  ______________________________________|_________________________|
    //    | |                                      |
    //    | |                                      a parameter
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
                assert!(next_start_vertical_offset < end_offset_for_start, "assertion failed: next_start_vertical_offset < end_offset_for_start\n\
                next_start_vertical_offset: {}, end_offset_for_start: {}, static_offset_from_start: {}", next_start_vertical_offset, end_offset_for_start, static_offset_from_start);

                vertical_offsets[i] = next_start_vertical_offset;
                next_start_vertical_offset += 1;
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

fn calculate_final_data<FileId: Copy>(diagnostic: &Diagnostic<FileId>, files: &impl Files<FileId=FileId>, file: FileId,
                                      line_index: usize,
                                      starts_ends: &[(&Annotation<FileId>, StartEndAnnotationData)],
                                      mut vertical_offsets: Vec<u32>,
                                      continuing_annotations: &[&Annotation<FileId>]) -> Result<Vec<Vec<AnnotationData>>, Error> {
    // Create a sorted vector with the vertical offsets (and an index into starts_ends)
    let mut vertical_offsets_sorted = vertical_offsets.iter().enumerate()
        .map(|(i, offset)| (i, *offset)).collect::<Vec<_>>();
    vertical_offsets_sorted.sort_by(|(_, a), (_, b)| a.cmp(b)); // sort by the vertical offset
    // How many elements from the start of continuing_annotations to take
    // len() - 1 could cause an underflow (panic), and take() stops when the end of the iter is reached anyway
    let mut continuing_take_index: usize = continuing_annotations.len();

    for (i, a) in continuing_annotations.iter().enumerate().rev() {
        // Once we reach a continuing annotation that started before this line,
        // all the ones before it in the vector should start before too, so we can stop here
        // and use i as the last index to use for the continuing vertical bars on the first line
        if files.line_index(file, a.range.start)? < line_index {
            continuing_take_index = i;
            break;
        }
    }

    // the last vertical index; can be used to estimate how many lines are needed for
    // displaying the annotations.
    // This is not exact, as there can be extra lines for labels, as one example.
    let _final_vertical_index = vertical_offsets_sorted.last().map(|(_, offset)| *offset).unwrap_or(1);

    // Create ContinuingMultiline data for the continuing vertical bars at the start.
    let mut data = continuing_annotations.iter().take(continuing_take_index)
        .fold(Vec::new(), |mut acc, a| {
            acc.push(AnnotationData::ContinuingMultiline(ContinuingMultilineAnnotationData {
                style: a.style,
                severity: diagnostic.severity,
                vertical_bar_index: acc.len(),
            }));
            acc
        });

    // Add a ConnectingMultiline element if needed (if there is an ending multi-line annotation with vertical offset == 0)
    // this is the horizontal "_____" line running from the continuing vertical bar to the location
    // that the annotations ends at
    if let Some((i, _)) = vertical_offsets.iter().enumerate().find(|(i, &offset)| offset == 0 && match starts_ends[*i].1 {
        StartEndAnnotationData::End(_) => true,
        StartEndAnnotationData::Start(_) | StartEndAnnotationData::Both(_, _) => false,
    }) {
        let a = starts_ends[i].0;

        data.push(AnnotationData::ConnectingMultiline(ConnectingMultilineAnnotationData {
            style: a.style,
            severity: diagnostic.severity,
            end_location: LineColumn::new(line_index, a.range.end - files.line_range(file, line_index)?.start),
            // All elements in data so far are ContinuingMultiline, so this one must connect to the last one
            vertical_bar_index: data.len() - 1,
        }))
    }

    // Add the start and end boundary and single-line connecting annotation data (the "^^^^^^^^^")
    let mut data = vertical_offsets.iter().enumerate().fold(data, |mut acc, (i, _)| {
        let (annotation, start_end) = &starts_ends[i];

        match start_end {
            StartEndAnnotationData::Start(start) => {
                // A single start boundary marker. This should either have a connecting element
                // either in this line or on a later one (with hanging elements ("|") in between)
                acc.push(AnnotationData::Start(start.clone()));
            },
            StartEndAnnotationData::End(end) => {
                // Same here
                acc.push(AnnotationData::End(end.clone()));
            },
            StartEndAnnotationData::Both(start, end) => {
                // Add start and end boundary elements and the connecting line between them.
                // They all have the same character, so they will be rendered as a single line:
                // "^^^^^^^^^" or "---------"
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

    if vertical_offsets[starts_ends.len() - 1] == 0 {
        let (a, start_end) = &starts_ends[starts_ends.len() - 1];

        let label_pos = match start_end {
            StartEndAnnotationData::End(end) => Some(end.location.column_index),
            StartEndAnnotationData::Both(_, end) => Some(end.location.column_index),
            StartEndAnnotationData::Start(_) => None,
        };
        let has_label = label_pos.is_some() && !a.label.is_empty();

        if let (true, Some(label_pos)) = (has_label, label_pos) {
            data.push(AnnotationData::Label(LabelAnnotationLineData {
                style: a.style,
                severity: diagnostic.severity,
                location: LineColumn::new(line_index, label_pos + 1),
                label: a.label.clone(),
            }));
        }
    }

    // At which vertical index we currently are (should correspond to vertical offset of the annotations)
    let mut vertical_index = 1; // first line after the one with the underlines
    let mut additional_continuing_indices = Vec::new(); // controlled by calculate_single_line_data()
    let mut final_data = vec![data];

    for (_i, offset) in vertical_offsets_sorted.iter() {
        let vertical_offset = *offset;

        if vertical_offset > vertical_index {
            vertical_index = vertical_offset;
            final_data.push(calculate_single_line_data(diagnostic, files, file, line_index, vertical_index,
                continuing_annotations, continuing_take_index, &mut additional_continuing_indices,
                starts_ends, &mut vertical_offsets)?);
        }
    }

    Ok(final_data)
}

fn calculate_single_line_data<FileId: Copy>(diagnostic: &Diagnostic<FileId>, files: &impl Files<FileId=FileId>, file: FileId,
                                            line_index: usize, vertical_index: u32,
                                            continuing_annotations: &[&Annotation<FileId>], continuing_take_index: usize,
                                            additional_continuing_indices: &mut Vec<usize>,
                                            starts_ends: &[(&Annotation<FileId>, StartEndAnnotationData)],
                                            vertical_offsets: &mut [u32]) -> Result<Vec<AnnotationData>, Error> {
    // Create ContinuingMultiline data for the continuing vertical bars at the start.
    let mut data = continuing_annotations.iter().take(continuing_take_index)
        .fold(Vec::new(), |mut acc, a| {
            acc.push(AnnotationData::ContinuingMultiline(ContinuingMultilineAnnotationData {
                style: a.style,
                severity: diagnostic.severity,
                vertical_bar_index: acc.len(),
            }));
            acc
        });

    for index in additional_continuing_indices.iter() {
        let (annotation, _) = &starts_ends[*index];

        data.push(AnnotationData::ContinuingMultiline(ContinuingMultilineAnnotationData {
            style: annotation.style,
            severity: diagnostic.severity,
            vertical_bar_index: data.len(),
        }));
    }

    // Add a ConnectingMultiline element if needed (if there is an ending multi-line annotation with vertical offset == 0)
    // this is the horizontal "_____" line running from the continuing vertical bar to the location
    // that the annotations ends at
    if let Some((i, _)) = vertical_offsets.iter().enumerate().find(|(i, &offset)| offset == vertical_index && match starts_ends[*i].1 {
        StartEndAnnotationData::End(_) => true,
        StartEndAnnotationData::Start(_) | StartEndAnnotationData::Both(_, _) => false,
    }) {
        let a = starts_ends[i].0;

        data.push(AnnotationData::ConnectingMultiline(ConnectingMultilineAnnotationData {
            style: a.style,
            severity: diagnostic.severity,
            end_location: LineColumn::new(line_index, a.range.end - files.line_range(file, line_index)?.start),
            // All elements in data so far are ContinuingMultiline, so this one must connect to the last one
            vertical_bar_index: data.len() - 1,
        }))
    }

    let mut push_down_end = None;

    // This does different things depending on the vertical_index:
    // If vertical_index is 0:
    //   add the start and end boundary and single-line connecting annotation data (the "^^^^^^^^^")
    // If vertical_index > 0:
    //   add the hanging annotation data (the "|" over labels or for connecting lines)
    //
    // In both cases, in can also add connecting lines.
    let mut data = vertical_offsets.iter().enumerate().fold(data, |mut acc, (i, &offset)| {
        let (annotation, start_end) = &starts_ends[i];

        match start_end {
            StartEndAnnotationData::Start(start) => {
                if offset == vertical_index {
                    // If this is the line this annotation should connect with its
                    // continuing vertical bar, add the connection line
                    acc.push(AnnotationData::ConnectingMultiline(ConnectingMultilineAnnotationData {
                        style: annotation.style,
                        severity: diagnostic.severity,
                        end_location: start.location.clone(),
                        vertical_bar_index: continuing_annotations.len() - continuing_take_index + additional_continuing_indices.len(),
                    }));
                    additional_continuing_indices.push(i);

                    push_down_end = Some(vertical_index);
                }

                if vertical_index == 0 {
                    // A single start boundary marker. This should either have a connecting element
                    // either in this line or on a later one (with hanging elements ("|") in between)
                    acc.push(AnnotationData::Start(start.clone()));
                } else if offset >= vertical_index {
                    // If vertical_index is not at offset yet, and we're not on the line that
                    // should have the boundary marker, add a "|" character
                    acc.push(AnnotationData::Hanging(HangingAnnotationLineData {
                        style: annotation.style,
                        severity: diagnostic.severity,
                        location: start.location.clone(),
                    }));
                }
            },
            StartEndAnnotationData::End(end) => {
                if offset == vertical_index {
                    // If this is the line this annotation should connect with its
                    // continuing vertical bar, add the connection line
                    acc.push(AnnotationData::ConnectingMultiline(ConnectingMultilineAnnotationData {
                        style: annotation.style,
                        severity: diagnostic.severity,
                        end_location: end.location.clone(),
                        vertical_bar_index: continuing_annotations.len() - continuing_take_index + additional_continuing_indices.len(),
                    }));
                    additional_continuing_indices.push(i);
                }

                if vertical_index == 0 {
                    acc.push(AnnotationData::End(end.clone()));
                } else {
                    if offset + 1 == vertical_index && !annotation.label.is_empty() {
                        // If we're just under the continuing line and this annotation has a label, add it.
                        //
                        // TODO intersecting lines with further starting annotations
                        // should be able to push this further down (add an additional vertical offset)
                        acc.push(AnnotationData::Label(LabelAnnotationLineData {
                            style: annotation.style,
                            severity: diagnostic.severity,
                            location: end.location.clone(),
                            label: annotation.label.clone(),
                        }));
                    } else if offset >= vertical_index {
                        // If vertical_index is not at offset yet, and we're not on the line that
                        // should have the boundary marker, add a "|" character
                        acc.push(AnnotationData::Hanging(HangingAnnotationLineData {
                            style: annotation.style,
                            severity: diagnostic.severity,
                            location: end.location.clone(),
                        }));
                    }
                }
            },
            StartEndAnnotationData::Both(start, end) => {
                if vertical_index == 0 {
                    // Add start and end boundary elements and the connecting line between them.
                    // They all have the same character, so they will be rendered as a single line:
                    // "^^^^^^^^^" or "---------"
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
                } else {
                    if offset + 1 == vertical_index && !annotation.label.is_empty() {
                        // If we're under the hanging elements ("|") and this annotation has a label, add it.
                        //
                        // TODO intersecting lines with further starting annotations
                        //      should be able to push this further down (add an additional vertical offset)
                        acc.push(AnnotationData::Label(LabelAnnotationLineData {
                            style: annotation.style,
                            severity: diagnostic.severity,
                            location: start.location.clone(),
                            label: annotation.label.clone(),
                        }));
                    } else if offset >= vertical_index {
                        // If vertical_index is not at offset yet, and we're not on the line that
                        // should have the boundary marker, add a "|" character
                        acc.push(AnnotationData::Hanging(HangingAnnotationLineData {
                            style: annotation.style,
                            severity: diagnostic.severity,
                            location: start.location.clone(),
                        }));
                    }
                }
            },
        };

        acc
    });

    if let Some(to_offset) = push_down_end {
        let mut next_vertical_offset = to_offset + 2;

        for (i, offset) in vertical_offsets.iter_mut().enumerate() {
            let (a, start_end) = &starts_ends[i];

            match start_end {
                // end and both, which should be below start, need to be moved down
                StartEndAnnotationData::End(_) | StartEndAnnotationData::Both(_, _) => {},
                // don't affect starting annotations
                StartEndAnnotationData::Start(_) => continue,
            }

            if *offset <= to_offset && !a.label.is_empty() {
                // It can't be equal, because we would have a starting annotation then
                assert_ne!(*offset, to_offset);

                *offset = next_vertical_offset;
                next_vertical_offset += 1;
            }
        }
    }

    // TODO Hasn't been adjusted to the code being moved into a function
    //      that also runs on vertical indices > 0
    // Actually, maybe this just works anyway
    if vertical_index == 0 && vertical_offsets[starts_ends.len() - 1] == 0 {
        let (a, start_end) = &starts_ends[starts_ends.len() - 1];

        let label_pos = match start_end {
            StartEndAnnotationData::End(end) => Some(end.location.column_index),
            StartEndAnnotationData::Both(_, end) => Some(end.location.column_index),
            StartEndAnnotationData::Start(_) => None,
        };
        let has_label = label_pos.is_some() && !a.label.is_empty();

        if let (true, Some(label_pos)) = (has_label, label_pos) {
            data.push(AnnotationData::Label(LabelAnnotationLineData {
                style: a.style,
                severity: diagnostic.severity,
                location: LineColumn::new(line_index, label_pos + 2),
                label: a.label.clone(),
            }));
        }
    }

    Ok(data)
}

#[cfg(test)]
mod tests;
