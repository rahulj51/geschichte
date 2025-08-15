use crate::error::{GeschichteError, Result};

/// Parses git log output into structured data
pub fn parse_log_line(line: &str) -> Result<(String, String, String, String, String)> {
    let parts: Vec<&str> = line.split('\0').collect();
    if parts.len() != 5 {
        return Err(GeschichteError::ParseError {
            reason: format!("Expected 5 fields, got {}", parts.len()),
        });
    }
    
    Ok((
        parts[0].to_string(), // full hash
        parts[1].to_string(), // short hash
        parts[2].to_string(), // date
        parts[3].to_string(), // author
        parts[4].to_string(), // subject
    ))
}