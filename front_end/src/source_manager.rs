use std::collections::HashMap;
use crate::symbol_table::Span;

pub struct SourceManager {
    sources: HashMap<String, String>,
    default_source: String,
    line_starts: Vec<usize>,
}

impl SourceManager {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            default_source: String::new(),
            line_starts: vec![0],
        }
    }
    
    pub fn add_source(&mut self, name: &str, content: &str) {
        self.sources.insert(name.to_string(), content.to_string());
    }
    
    pub fn set_default_source(&mut self, content: &str) {
        self.default_source = content.to_string();
        self.line_starts = vec![0]; // Reset line starts, first line starts at position 0
        
        // Find all line start positions (after newlines)
        for (i, c) in content.char_indices() {
            if c == '\n' {
                self.line_starts.push(i + 1);
            }
        }
    }
    
    // Get a specific line from the source
    pub fn get_line(&self, line_number: usize) -> Option<&str> {
        if line_number == 0 || line_number > self.line_starts.len() {
            return None;
        }
        
        let start = self.line_starts[line_number - 1]; // Line numbers are 1-based
        let end = if line_number < self.line_starts.len() {
            self.line_starts[line_number]
        } else {
            self.default_source.len()
        };
        
        Some(&self.default_source[start..end])
    }
    
    // Get a snippet for a specific span with context
    pub fn get_snippet(&self, span: &Span) -> String {
        // Handle out of bounds
        if span.start_line == 0 || span.start_line > self.line_starts.len() {
            return String::from("<invalid line number>");
        }
        
        // Get the source line
        let line = self.get_line(span.start_line).unwrap_or("<line not found>");
        let trimmed_line = line.trim_end();
        
        // Calculate the indentation level
        let indent_size = line.len() - line.trim_start().len();
        
        // Create the caret line (^^^^^)
        let mut carets = String::new();
        
        // First fill with spaces up to the variable position, accounting for indentation
        // The key is to adjust for the indentation we're showing in the output
        // We need to add spaces equal to the column minus 1 (0-based indexing)
        for _ in 0..(span.start_column - indent_size + 4) {
            carets.push(' ');
        }
        
        // Add the caret
        carets.push('^');
        
        // Return the line and caret indicator
        format!("    {}\n{}", trimmed_line.trim_start(), carets)
    }
}