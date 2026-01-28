/// Nanopore FASTQ header parser and types
/// Example header:
/// @read_id runid=abcd1234 ch=123 start_time=2020-01-01T00:00:00Z
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NanoporeHeader {
    pub read_id: String,
    pub attributes: HashMap<String, String>, // Flexible k=v pairs
}

impl NanoporeHeader {
    pub fn parse(header_line: &str) -> Option<Self> {
        let parts: Vec<&str> = header_line.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let read_id = parts[0].trim_start_matches('@').to_string();
        let mut attrs = HashMap::new();

        for part in &parts[1..] {
            if let Some((k, v)) = part.split_once('=') {
                attrs.insert(k.to_string(), v.to_string());
            }
        }

        Some(NanoporeHeader {
            read_id,
            attributes: attrs,
        })
    }

    pub fn get_runid(&self) -> Option<&str> {
        self.attributes.get("runid").map(|s| s.as_str())
    }

    pub fn get_ch(&self) -> Option<&str> {
        self.attributes.get("ch").map(|s| s.as_str())
    }
}
