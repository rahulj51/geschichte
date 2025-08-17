use super::{DiffLine, DiffLineType};

#[derive(Debug, Clone)]
pub struct SideBySideDiff {
    pub old_lines: Vec<Option<DiffLine>>,
    pub new_lines: Vec<Option<DiffLine>>,
}

impl SideBySideDiff {
    /// Convert a unified diff into side-by-side view
    pub fn from_unified(diff_lines: &[DiffLine]) -> Self {
        let mut old_lines = Vec::new();
        let mut new_lines = Vec::new();
        
        for line in diff_lines {
            match line.line_type {
                DiffLineType::Header | DiffLineType::HunkHeader => {
                    // Headers appear in both sides
                    old_lines.push(Some(line.clone()));
                    new_lines.push(Some(line.clone()));
                }
                DiffLineType::Context => {
                    // Context lines appear in both sides
                    old_lines.push(Some(line.clone()));
                    new_lines.push(Some(line.clone()));
                }
                DiffLineType::Deletion => {
                    // Deletion only appears in old file
                    old_lines.push(Some(line.clone()));
                    new_lines.push(None); // Placeholder for alignment
                }
                DiffLineType::Addition => {
                    // Addition only appears in new file
                    old_lines.push(None); // Placeholder for alignment
                    new_lines.push(Some(line.clone()));
                }
            }
        }
        
        // Compact consecutive additions and deletions for better visual alignment
        Self::compact_changes(&mut old_lines, &mut new_lines);
        
        Self {
            old_lines,
            new_lines,
        }
    }
    
    /// Compact consecutive additions and deletions to align them side by side
    fn compact_changes(
        old_lines: &mut Vec<Option<DiffLine>>,
        new_lines: &mut Vec<Option<DiffLine>>,
    ) {
        // This is a simplified version - a more sophisticated algorithm would
        // better align changes based on content similarity
        
        let mut i = 0;
        while i < old_lines.len() {
            // Find a deletion followed by additions
            if old_lines[i].is_some() && new_lines[i].is_none() {
                if let Some(ref line) = old_lines[i] {
                    if line.line_type == DiffLineType::Deletion {
                        // Look for following additions
                        let mut j = i + 1;
                        while j < old_lines.len() 
                            && old_lines[j].is_none() 
                            && new_lines[j].is_some() {
                            if let Some(ref new_line) = new_lines[j] {
                                if new_line.line_type != DiffLineType::Addition {
                                    break;
                                }
                            }
                            j += 1;
                        }
                        
                        // We have deletions from i to some point, and additions after
                        // Compact them to be side by side
                        let num_additions = j - i - 1;
                        
                        if num_additions > 0 {
                            // Move the first addition to align with the deletion
                            if i + 1 < new_lines.len() {
                                new_lines.swap(i, i + 1);
                                // Remove the now-empty line
                                if i + 1 < old_lines.len() && old_lines[i + 1].is_none() && new_lines[i + 1].is_none() {
                                    old_lines.remove(i + 1);
                                    new_lines.remove(i + 1);
                                }
                            }
                        }
                    }
                }
            }
            i += 1;
        }
    }
}