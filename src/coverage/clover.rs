use std::io::{BufReader, Error, ErrorKind, Read, Result};
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use super::{CoverageParser, CoverageSink};
use crate::util::path::normalize_path;

pub struct CloverParser;

impl CoverageParser for CloverParser {
    fn can_parse<R: Read>(&self, reader: R) -> Result<bool> {
        let mut limited = reader.take(8192);
        let mut buf = String::new();
        limited.read_to_string(&mut buf).map_err(|err| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Failed to read Clover XML: {err}"),
            )
        })?;
        let haystack = buf.as_str();

        let has_file = haystack.contains("<file");
        let has_count = haystack.contains("count=");
        let has_num = haystack.contains("num=");

        Ok(has_file && (has_count || has_num))
    }

    fn parse<R: Read>(&self, reader: R, sink: &mut dyn CoverageSink) -> Result<()> {
        let mut xml = Reader::from_reader(BufReader::new(reader));
        xml.config_mut().trim_text(true);
        let mut buf = Vec::new();
        let mut current_file: Option<String> = None;

        loop {
            match xml.read_event_into(&mut buf) {
                Ok(Event::Start(event)) => match event.name().as_ref() {
                    b"file" => {
                        if let Some(path) =
                            read_attribute_any(&event, &[b"name", b"path", b"filename"])?
                        {
                            let normalized = normalize_coverage_path(&path);
                            current_file = Some(normalized.clone());
                            sink.on_file(&normalized);
                        }
                    }
                    b"line" => {
                        if let Some(file_path) = current_file.as_deref() {
                            if let Some((number, hits)) = read_line_attributes(&event)? {
                                sink.on_line(file_path, number, hits);
                            }
                        }
                    }
                    _ => {}
                },
                Ok(Event::Empty(event)) => match event.name().as_ref() {
                    b"file" => {
                        if let Some(path) =
                            read_attribute_any(&event, &[b"name", b"path", b"filename"])?
                        {
                            let normalized = normalize_coverage_path(&path);
                            sink.on_file(&normalized);
                        }
                    }
                    b"line" => {
                        if let Some(file_path) = current_file.as_deref() {
                            if let Some((number, hits)) = read_line_attributes(&event)? {
                                sink.on_line(file_path, number, hits);
                            }
                        }
                    }
                    _ => {}
                },
                Ok(Event::End(event)) => {
                    if event.name().as_ref() == b"file" {
                        current_file = None;
                    }
                }
                Ok(Event::Eof) => break,
                Err(err) => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Failed to parse Clover XML: {err}"),
                    ))
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(())
    }
}

fn read_attribute_any(
    event: &quick_xml::events::BytesStart<'_>,
    keys: &[&[u8]],
) -> Result<Option<String>> {
    for attr in event.attributes() {
        let attr = attr.map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
        if keys.iter().any(|key| attr.key.as_ref() == *key) {
            let value = attr
                .unescape_value()
                .map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
            return Ok(Some(value.into_owned()));
        }
    }
    Ok(None)
}

fn read_line_attributes(event: &quick_xml::events::BytesStart<'_>) -> Result<Option<(u32, u32)>> {
    let mut number: Option<u32> = None;
    let mut hits: Option<u32> = None;
    let mut line_type: Option<String> = None;

    for attr in event.attributes() {
        let attr = attr.map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
        match attr.key.as_ref() {
            b"num" | b"number" => {
                let value = attr
                    .unescape_value()
                    .map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
                number = parse_u32_lossy(&value);
            }
            b"count" | b"hits" => {
                let value = attr
                    .unescape_value()
                    .map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
                hits = parse_u32_lossy(&value);
            }
            b"type" => {
                let value = attr
                    .unescape_value()
                    .map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
                line_type = Some(value.into_owned());
            }
            _ => {}
        }
    }

    if let Some(line_type) = line_type.as_deref() {
        if line_type == "method" {
            return Ok(None);
        }
    }

    match (number, hits) {
        (Some(number), Some(hits)) => Ok(Some((number, hits))),
        _ => Ok(None),
    }
}

fn normalize_coverage_path(path: &str) -> String {
    let path_obj = Path::new(path);
    if path_obj.is_absolute() {
        if let Ok(cwd) = std::env::current_dir() {
            if let Ok(stripped) = path_obj.strip_prefix(&cwd) {
                return normalize_path(&stripped.to_string_lossy());
            }
        }
    }
    normalize_path(path)
}

fn parse_u32_lossy(value: &str) -> Option<u32> {
    value.parse().ok().or_else(|| {
        value
            .parse::<f64>()
            .ok()
            .map(|parsed| parsed.max(0.0) as u32)
    })
}

#[cfg(test)]
mod tests {
    use super::CloverParser;
    use crate::coverage::store::CoverageStore;
    use crate::coverage::CoverageParser;
    use std::io::Cursor;

    #[test]
    fn parses_statement_counts() {
        let file = Cursor::new(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/coverage_clover.xml"
        )));
        let mut store = CoverageStore::default();
        CloverParser.parse(file, &mut store).expect("parse clover");

        let coverage = store
            .file_coverage("src/Calculator.php")
            .expect("lookup")
            .expect("file coverage");
        assert!(coverage.is_covered(11));
        assert!(!coverage.is_covered(16));
    }

    #[test]
    fn matches_changed_lines_from_diff() {
        let diff_reader = std::io::BufReader::new(Cursor::new(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/git_diff.diff"
        ))));
        let changed = crate::diff::git::parse_unified_diff(diff_reader).expect("parse diff");

        let cov_file = Cursor::new(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/coverage_clover.xml"
        )));
        let mut store = CoverageStore::default();
        CloverParser
            .parse(cov_file, &mut store)
            .expect("parse clover");

        let report = crate::coverage::analyze_changed_coverage(&changed, &store, true);
        let report = report.expect("report");
        let calc = report
            .uncovered_files
            .iter()
            .find(|file| file.path.ends_with("src/Calculator.php"))
            .expect("calculator coverage");
        assert!(!calc.uncovered_lines.contains(&11));
    }

    #[test]
    fn ignores_method_lines() {
        let file = Cursor::new(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/coverage_clover.xml"
        )));
        let mut store = CoverageStore::default();
        CloverParser.parse(file, &mut store).expect("parse clover");

        let coverage = store
            .file_coverage("src/Calculator.php")
            .expect("lookup")
            .expect("file coverage");
        assert!(!coverage.is_covered(9));
        assert!(coverage.is_covered(11));
    }

    #[test]
    fn can_parse_clover_fixture() {
        let file = Cursor::new(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/coverage_clover.xml"
        )));
        assert!(CloverParser.can_parse(file).expect("detect clover"));
    }

    #[test]
    fn does_not_parse_cobertura_fixture() {
        let file = Cursor::new(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/coverage_cobertura.xml"
        )));
        assert!(!CloverParser.can_parse(file).expect("detect cobertura"));
    }
}
