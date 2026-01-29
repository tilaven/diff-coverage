use std::io::{BufReader, Error, ErrorKind, Read, Result};
use std::path::Path;

use super::{CoverageParser, CoverageSink};
use crate::util::path::normalize_path;
use quick_xml::events::Event;
use quick_xml::Reader;

pub struct CoberturaParser;

impl CoverageParser for CoberturaParser {
    fn can_parse<R: Read>(&self, reader: R) -> Result<bool> {
        let mut limited = reader.take(8192);
        let mut buf = String::new();
        limited.read_to_string(&mut buf).map_err(|err| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Failed to read Cobertura XML: {err}"),
            )
        })?;
        let haystack = buf.as_str();

        let has_cobertura = haystack.contains("cobertura");
        let has_class = haystack.contains("<class");
        let has_filename = haystack.contains("filename=");
        let has_line_hits = haystack.contains("line number=") && haystack.contains("hits=");

        Ok(has_cobertura || (has_class && has_filename) || has_line_hits)
    }

    fn parse<R: Read>(&self, reader: R, sink: &mut dyn CoverageSink) -> Result<()> {
        let mut xml = Reader::from_reader(BufReader::new(reader));
        xml.config_mut().trim_text(true);
        let mut buf = Vec::new();
        let mut current_file: Option<String> = None;

        loop {
            match xml.read_event_into(&mut buf) {
                Ok(Event::Start(event)) => match event.name().as_ref() {
                    b"class" => {
                        if let Some(path) = read_attribute(&event, b"filename")? {
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
                    b"class" => {
                        if let Some(path) = read_attribute(&event, b"filename")? {
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
                    if event.name().as_ref() == b"class" {
                        current_file = None;
                    }
                }
                Ok(Event::Eof) => break,
                Err(err) => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Failed to parse Cobertura XML: {err}"),
                    ))
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(())
    }
}

fn read_attribute(event: &quick_xml::events::BytesStart<'_>, key: &[u8]) -> Result<Option<String>> {
    for attr in event.attributes() {
        let attr = attr.map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
        if attr.key.as_ref() == key {
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

    for attr in event.attributes() {
        let attr = attr.map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
        match attr.key.as_ref() {
            b"number" => {
                let value = attr
                    .unescape_value()
                    .map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
                number = value.parse().ok();
            }
            b"hits" => {
                let value = attr
                    .unescape_value()
                    .map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
                hits = value.parse().ok();
            }
            _ => {}
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

#[cfg(test)]
mod tests {
    use super::CoberturaParser;
    use crate::coverage::store::CoverageStore;
    use crate::coverage::CoverageParser;
    use std::io::Cursor;

    #[test]
    fn can_parse_cobertura_fixture() {
        let file = Cursor::new(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/coverage_cobertura.xml"
        )));
        assert!(CoberturaParser.can_parse(file).expect("detect cobertura"));
    }

    #[test]
    fn does_not_parse_clover_fixture() {
        let file = Cursor::new(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/coverage_clover.xml"
        )));
        assert!(!CoberturaParser.can_parse(file).expect("detect clover"));
    }

    #[test]
    fn parses_line_hits_from_fixture() {
        let file = Cursor::new(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/coverage_cobertura.xml"
        )));
        let mut store = CoverageStore::default();
        CoberturaParser
            .parse(file, &mut store)
            .expect("parse cobertura");

        let coverage = store
            .file_coverage("Calculator.php")
            .expect("lookup")
            .expect("file coverage");
        assert_eq!(coverage.measured_lines, vec![11, 16]);
        assert!(coverage.covered_lines.is_empty());
    }
}
