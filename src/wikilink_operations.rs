use pathdiff::diff_paths;
use serde_yaml::Value;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

// Replaces wikilinks in the given text with their corresponding reference in entries.
pub fn find_wikilinks(text: &str, entries: &[String], entry: &str, path_prefix: &str) -> String {
    let re = regex::Regex::new(r"\[\[(.+?)\]\]").unwrap();
    re.replace_all(text, |caps: &regex::Captures| {
        let reference = caps.get(1).unwrap().as_str();
        find_reference(reference, entries, entry, path_prefix)
    })
    .to_string()
}

// Finds the corresponding reference in entries and returns the properly formatted link.
pub fn find_reference(
    reference: &str,
    entries: &[String],
    entry_path: &str,
    path_prefix: &str,
) -> String {
    let matching_entry = entries.iter().find(|entry| {
        let filename = Path::new(entry).file_stem().unwrap().to_str().unwrap();
        filename == reference
    });

    match matching_entry {
        Some(entry) => format_link(entry, reference, entry_path, path_prefix),
        None => format!("[{}](/blog/)", reference),
    }
}

pub fn parse_yaml_frontmatter(path: &str) -> Result<Value, io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut yaml_lines = Vec::new();
    let mut in_yaml_block = false;

    for line in reader.lines() {
        let line = line?;
        if line == "---" {
            if in_yaml_block {
                break;
            } else {
                in_yaml_block = true;
            }
        } else if in_yaml_block {
            yaml_lines.push(line);
        }
    }

    let yaml_block: String = yaml_lines.join("\n");
    let parsed_yaml: Value = serde_yaml::from_str(&yaml_block).unwrap_or_default();

    Ok(parsed_yaml)
}

pub fn get_slug_from_yaml_frontmatter(value: serde_yaml::Value) -> String {
    if let Some(slug_value) = value.get("slug") {
        if let Some(slug_str) = slug_value.as_str() {
            return slug_str.to_string();
        }
    }
    String::new()
}

// Formats the link using the given entry and reference.
pub fn format_link(entry: &str, reference: &str, entry_path: &str, path_prefix: &str) -> String {
    let path_host = Path::new(entry_path);
    let path_reference = Path::new(entry);

    let rel_path = match diff_paths(path_reference, path_host) {
        Some(path) => path,
        None => {
            eprintln!("Error: Unable to compute relative path");
            return String::new();
        }
    };
    let rel_path = rel_path.strip_prefix("..").unwrap_or(&rel_path);
    let path = Path::new(&rel_path);
    let file_stem = path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .replace(' ', "%20");

    let yaml_frontmatter = parse_yaml_frontmatter(entry).unwrap_or_default();
    let slug = get_slug_from_yaml_frontmatter(yaml_frontmatter);

    format!(
        "[{}]({}/{})",
        reference,
        if !path_prefix.is_empty() {
            path_prefix
        } else {
            "./"
        },
        if !slug.is_empty() { slug } else { file_stem }
    )
}
