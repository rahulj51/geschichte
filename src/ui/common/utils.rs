use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

/// Apply horizontal scrolling to a line
pub fn apply_horizontal_scroll(
    line: Line<'static>,
    horizontal_offset: usize,
    viewport_width: usize,
) -> Line<'static> {
    // Calculate total line width in characters
    let total_width: usize = line
        .spans
        .iter()
        .map(|span| span.content.chars().count())
        .sum();

    // If no horizontal offset, return original line
    if horizontal_offset == 0 {
        return line;
    }

    // Always apply horizontal scrolling regardless of line length
    // This ensures visual alignment of all lines

    // If the horizontal offset is greater than the total line width,
    // return an empty line (the line is scrolled completely out of view)
    if horizontal_offset >= total_width {
        return Line::from(vec![]);
    }

    // Apply horizontal offset by trimming characters from the start
    let mut char_count = 0;
    let mut new_spans = Vec::new();
    let mut remaining_offset = horizontal_offset;

    for span in line.spans {
        let span_char_count = span.content.chars().count();

        if remaining_offset >= span_char_count {
            // Skip this entire span
            remaining_offset -= span_char_count;
            continue;
        }

        // Partial span - trim from the start
        let trimmed_content: String = span
            .content
            .chars()
            .skip(remaining_offset)
            .take(viewport_width.saturating_sub(char_count))
            .collect();

        if !trimmed_content.is_empty() {
            new_spans.push(Span::styled(trimmed_content.clone(), span.style));
            char_count += trimmed_content.chars().count();

            // Stop if we've filled the viewport
            if char_count >= viewport_width {
                break;
            }
        }

        remaining_offset = 0; // Used up the offset
    }

    Line::from(new_spans)
}

/// Create border style based on focus state
pub fn create_border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    }
}

/// Generate title for commits panel with optional horizontal scroll indicator
pub fn create_commits_title(
    commits_count: usize,
    loading: bool,
    horizontal_scroll: usize,
) -> String {
    let mut title = if loading {
        " Commits (Loading...) ".to_string()
    } else {
        format!(" Commits ({}) ", commits_count)
    };

    // Add horizontal scroll indicator
    if horizontal_scroll > 0 {
        title = format!("{} ←→", title.trim_end());
    }

    title
}

/// Generate title for diff panel with optional commit hash and range info
pub fn create_diff_title(
    commits: &[crate::commit::Commit],
    selected_index: usize,
    current_diff_range: Option<(usize, usize)>,
    diff_range_start: Option<usize>,
    horizontal_scroll: usize,
) -> String {
    let mut title = if commits.is_empty() {
        " Diff ".to_string()
    } else if selected_index < commits.len() {
        // Check if we're showing a range diff
        if let Some((older_idx, newer_idx)) = current_diff_range {
            if older_idx < commits.len() && newer_idx < commits.len() {
                format!(
                    " Diff ({}..{}) ",
                    commits[older_idx].short_hash, commits[newer_idx].short_hash
                )
            } else {
                format!(" Diff ({}) ", commits[selected_index].short_hash)
            }
        } else if let Some(_start_index) = diff_range_start {
            // Show that we're in selection mode
            format!(
                " Diff ({}) [Selecting...] ",
                commits[selected_index].short_hash
            )
        } else {
            format!(" Diff ({}) ", commits[selected_index].short_hash)
        }
    } else {
        " Diff ".to_string()
    };

    // Add horizontal scroll indicator
    if horizontal_scroll > 0 {
        title = format!("{} ←→", title.trim_end());
    }

    title
}

/// Generate title for side-by-side diff panels
pub fn create_side_by_side_title(
    commits: &[crate::commit::Commit],
    selected_index: usize,
    current_diff_range: Option<(usize, usize)>,
    is_old_file: bool,
) -> String {
    if commits.is_empty() {
        return if is_old_file {
            " Old File ".to_string()
        } else {
            " New File ".to_string()
        };
    }

    // Check if we're showing a range diff
    if let Some((older_idx, newer_idx)) = current_diff_range {
        if older_idx < commits.len() && newer_idx < commits.len() {
            if is_old_file {
                format!(" Old ({}) ", commits[older_idx].short_hash)
            } else {
                format!(" New ({}) ", commits[newer_idx].short_hash)
            }
        } else if is_old_file {
            " Old File ".to_string()
        } else {
            " New File ".to_string()
        }
    } else {
        // For single commit diff, show the same commit hash as unified layout does
        // This represents "the diff OF this commit" not "diff FROM parent TO commit"
        if is_old_file {
            format!(" Old ({}) ", commits[selected_index].short_hash)
        } else {
            format!(" New ({}) ", commits[selected_index].short_hash)
        }
    }
}
