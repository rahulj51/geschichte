use once_cell::sync::Lazy;
use ratatui::style::{Color as RatatuiColor, Modifier};
use ratatui::text::Span;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::{SyntaxReference, SyntaxSet};

/// Lazy-loaded syntax definitions
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);

/// Lazy-loaded themes
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

/// Highlight a line of code for a given file path
pub fn highlight_line(line: &str, file_path: &Path) -> Vec<Span<'static>> {
    let syntax = detect_syntax(file_path);

    if let Some(syntax) = syntax {
        let theme = &THEME_SET.themes["InspiredGitHub"]; // Try a light theme designed for GitHub
        let mut highlighter = HighlightLines::new(syntax, theme);

        match highlighter.highlight_line(line, &SYNTAX_SET) {
            Ok(ranges) => {
                ranges
                    .into_iter()
                    .map(|(style, text)| {
                        let color = syntect_to_ratatui_color(style.foreground);
                        let mut ratatui_style = ratatui::style::Style::default().fg(color);

                        // Add modifiers based on syntect style
                        if style
                            .font_style
                            .contains(syntect::highlighting::FontStyle::BOLD)
                        {
                            ratatui_style = ratatui_style.add_modifier(Modifier::BOLD);
                        }
                        if style
                            .font_style
                            .contains(syntect::highlighting::FontStyle::ITALIC)
                        {
                            ratatui_style = ratatui_style.add_modifier(Modifier::ITALIC);
                        }
                        if style
                            .font_style
                            .contains(syntect::highlighting::FontStyle::UNDERLINE)
                        {
                            ratatui_style = ratatui_style.add_modifier(Modifier::UNDERLINED);
                        }

                        Span::styled(text.to_string(), ratatui_style)
                    })
                    .collect()
            }
            Err(_) => {
                // Fallback to plain text
                vec![Span::raw(line.to_string())]
            }
        }
    } else {
        // No syntax highlighting available
        vec![Span::raw(line.to_string())]
    }
}

/// Detect syntax from file extension
fn detect_syntax(file_path: &Path) -> Option<&'static SyntaxReference> {
    let extension = file_path.extension()?.to_str()?;

    // Try to find syntax by extension
    SYNTAX_SET
        .find_syntax_by_extension(extension)
        .or_else(|| {
            // Fallback to file name
            let file_name = file_path.file_name()?.to_str()?;
            SYNTAX_SET.find_syntax_by_name(file_name)
        })
        .or_else(|| {
            // Special cases
            match extension {
                "rs" => SYNTAX_SET.find_syntax_by_name("Rust"),
                "py" => SYNTAX_SET.find_syntax_by_name("Python"),
                "js" | "jsx" => SYNTAX_SET.find_syntax_by_name("JavaScript"),
                "ts" | "tsx" => SYNTAX_SET.find_syntax_by_name("TypeScript"),
                "md" => SYNTAX_SET.find_syntax_by_name("Markdown"),
                "toml" => SYNTAX_SET.find_syntax_by_name("TOML"),
                "yaml" | "yml" => SYNTAX_SET.find_syntax_by_name("YAML"),
                "json" => SYNTAX_SET.find_syntax_by_name("JSON"),
                "sh" | "bash" => SYNTAX_SET.find_syntax_by_name("Bash"),
                "go" => SYNTAX_SET.find_syntax_by_name("Go"),
                "cpp" | "cc" | "cxx" => SYNTAX_SET.find_syntax_by_name("C++"),
                "c" | "h" => SYNTAX_SET.find_syntax_by_name("C"),
                _ => None,
            }
        })
}

/// Convert syntect color to ratatui color
fn syntect_to_ratatui_color(color: syntect::highlighting::Color) -> RatatuiColor {
    RatatuiColor::Rgb(color.r, color.g, color.b)
}
