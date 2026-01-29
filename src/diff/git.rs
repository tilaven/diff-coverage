use std::collections::HashMap;
use std::fmt;
use std::io::{self, BufRead};

use super::types::ChangedFile;

#[derive(Debug)]
pub enum DiffParseError {
    Io(io::Error),
    InvalidHunkHeader(String),
}

impl fmt::Display for DiffParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiffParseError::Io(err) => write!(f, "io error while reading diff: {err}"),
            DiffParseError::InvalidHunkHeader(line) => {
                write!(f, "invalid hunk header: {line}")
            }
        }
    }
}

impl std::error::Error for DiffParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DiffParseError::Io(err) => Some(err),
            DiffParseError::InvalidHunkHeader(_) => None,
        }
    }
}

impl From<io::Error> for DiffParseError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

pub type DiffParseResult<T> = Result<T, DiffParseError>;

pub fn parse_unified_diff<R: BufRead>(reader: R) -> DiffParseResult<Vec<ChangedFile>> {
    let mut files: HashMap<String, Vec<u32>> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut current_path: Option<String> = None;
    let mut in_hunk = false;
    let mut old_line: u32 = 0;
    let mut new_line: u32 = 0;

    for line_result in reader.lines() {
        let line = line_result?;

        if let Some(path) = line.strip_prefix("+++ ") {
            if path == "/dev/null" {
                current_path = None;
                continue;
            }

            let normalized = path.strip_prefix("b/").unwrap_or(path).to_string();

            if !files.contains_key(&normalized) {
                order.push(normalized.clone());
                files.insert(normalized.clone(), Vec::new());
            }
            current_path = Some(normalized);
            in_hunk = false;
            continue;
        }

        if line.starts_with("@@") {
            let Some(path) = current_path.as_ref() else {
                in_hunk = false;
                continue;
            };

            let (old_start, _old_count, new_start, _new_count) = parse_hunk_header(&line)
                .ok_or_else(|| DiffParseError::InvalidHunkHeader(line.clone()))?;

            old_line = old_start;
            new_line = new_start;
            in_hunk = true;
            if let Some(lines) = files.get_mut(path) {
                lines.reserve(4);
            }
            continue;
        }

        if !in_hunk {
            continue;
        }

        let Some(path) = current_path.as_ref() else {
            continue;
        };

        let Some(lines) = files.get_mut(path) else {
            continue;
        };

        if line.starts_with('+') && !line.starts_with("+++") {
            lines.push(new_line);
            new_line = new_line.saturating_add(1);
            continue;
        }

        if line.starts_with('-') && !line.starts_with("---") {
            old_line = old_line.saturating_add(1);
            continue;
        }

        if line.starts_with(' ') {
            old_line = old_line.saturating_add(1);
            new_line = new_line.saturating_add(1);
        }
    }

    let mut changed_files = Vec::with_capacity(order.len());
    for path in order {
        if let Some(lines) = files.remove(&path) {
            changed_files.push(ChangedFile {
                path,
                changed_lines: lines,
            });
        }
    }

    Ok(changed_files)
}

fn parse_hunk_header(line: &str) -> Option<(u32, u32, u32, u32)> {
    let body = line.strip_prefix("@@ ")?;
    let body = body.split(" @@").next()?;
    let mut parts = body.split_whitespace();
    let old_part = parts.next()?;
    let new_part = parts.next()?;
    let (old_start, old_count) = parse_range(old_part, '-')?;
    let (new_start, new_count) = parse_range(new_part, '+')?;
    Some((old_start, old_count, new_start, new_count))
}

fn parse_range(part: &str, prefix: char) -> Option<(u32, u32)> {
    let part = part.strip_prefix(prefix)?;
    let mut iter = part.split(',');
    let start = iter.next()?;
    let start: u32 = start.parse().ok()?;
    let count = match iter.next() {
        Some(value) => value.parse().ok()?,
        None => 1,
    };
    if start == 0 && count == 0 {
        return Some((start, count));
    }
    if start == 0 {
        return None;
    }
    Some((start, count))
}

#[cfg(test)]
mod tests {
    use super::parse_unified_diff;
    use std::io::Cursor;

    #[test]
    fn parses_added_lines_from_diff() {
        let diff = "\
diff --git a/src/lib.rs b/src/lib.rs
index 1111111..2222222 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,0 +1,1 @@
+pub fn c() {}
@@ -10,0 +12,2 @@
+pub fn d() {}
+pub fn e() {}
";
        let results = parse_unified_diff(Cursor::new(diff)).expect("parse diff");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "src/lib.rs");
        assert_eq!(results[0].changed_lines, vec![1, 12, 13]);
    }

    #[test]
    fn skips_deleted_files() {
        let diff = "\
diff --git a/src/old.rs b/src/old.rs
deleted file mode 100644
--- a/src/old.rs
+++ /dev/null
@@ -1,2 +0,0 @@
-fn gone() {}
-fn gone2() {}
";
        let results = parse_unified_diff(Cursor::new(diff)).expect("parse diff");
        assert!(results.is_empty());
    }

    #[test]
    fn changed_line() {
        let diff = r#"
diff --git a/hello.py b/hello.py
index 6f38b4d..1b9ca9c 100644
--- a/hello.py
+++ b/hello.py
@@ -1,2 +1,2 @@
  print "hello"
- print unknown_var
+ print unknown_var + test
        "#;
        let results = parse_unified_diff(Cursor::new(diff)).expect("parse diff");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "hello.py");
        assert_eq!(results[0].changed_lines, vec![2]);
    }

    #[test]
    fn removed_and_changed_line() {
        let diff = r#"
diff --git a/hello.py b/hello.py
index 6f38b4d..bac0d52 100644
--- a/hello.py
+++ b/hello.py
@@ -1,2 +1 @@
- print "hello"
- print unknown_var
+ print unknown_var + test
        "#;
        let results = parse_unified_diff(Cursor::new(diff)).expect("parse diff");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "hello.py");
        assert_eq!(results[0].changed_lines, vec![1]);
    }

    #[test]
    fn multiple_files() {
        let diff = r#"
diff --git a/hello.py b/hello.py
index b732142..b2ba069 100644
--- a/hello.py
+++ b/hello.py
@@ -1 +1,2 @@
 print "hello"
+print unknown_var
diff --git a/test.php b/test.php
index b3d9bbc..a766f11 100644
--- a/test.php
+++ b/test.php
@@ -1 +1,2 @@
 <?php
+declare(strict_types=1);
"#;
        let results = parse_unified_diff(Cursor::new(diff)).expect("parse diff");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].path, "hello.py");
        assert_eq!(results[0].changed_lines, vec![2]);
        assert_eq!(results[1].path, "test.php");
        assert_eq!(results[1].changed_lines, vec![2]);
    }
}
