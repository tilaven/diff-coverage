use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::coverage::CoverageParser;
use crate::{coverage, diff};

pub(crate) fn load_changed_files(path: &Path) -> Result<Vec<diff::types::ChangedFile>, String> {
    let file = std::fs::File::open(path).map_err(|err| err.to_string())?;
    let reader = std::io::BufReader::new(file);
    diff::git::parse_unified_diff(reader).map_err(|err| err.to_string())
}

pub(crate) fn load_coverage_files(
    paths: &[PathBuf],
) -> Result<coverage::store::CoverageStore, String> {
    let mut store = coverage::store::CoverageStore::default();
    for path in paths {
        load_coverage_file(path, &mut store)?;
    }
    store.prepare_lookup();
    Ok(store)
}

fn load_coverage_file(
    path: &Path,
    store: &mut coverage::store::CoverageStore,
) -> Result<(), String> {
    let cobertura = coverage::cobertura::CoberturaParser;
    if try_parse_coverage(&cobertura, path, store)? {
        return Ok(());
    }

    let clover = coverage::clover::CloverParser;
    if try_parse_coverage(&clover, path, store)? {
        return Ok(());
    }

    Err(format!(
        "No supported coverage parser matched the file {}",
        path.display()
    ))
}

fn try_parse_coverage<P: CoverageParser>(
    parser: &P,
    path: &Path,
    store: &mut coverage::store::CoverageStore,
) -> Result<bool, String> {
    let mut file = std::fs::File::open(path)
        .map_err(|err| format!("Failed to read coverage file {}: {err}", path.display()))?;
    let can_parse = parser.can_parse(&mut file).map_err(|err| err.to_string())?;
    if !can_parse {
        return Ok(false);
    }
    file.seek(SeekFrom::Start(0))
        .map_err(|err| format!("Failed to seek coverage file {}: {err}", path.display()))?;
    parser.parse(file, store).map_err(|err| err.to_string())?;
    Ok(true)
}

pub(crate) fn collect_coverage_files(coverage_paths: Vec<PathBuf>) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for path in coverage_paths {
        let metadata = std::fs::metadata(&path)
            .map_err(|err| format!("Failed to read coverage path {}: {err}", path.display()))?;
        if metadata.is_file() {
            files.push(path);
        } else if metadata.is_dir() {
            let mut dir_files = list_files_recursive(&path)?;
            files.append(&mut dir_files);
        } else {
            return Err(format!(
                "Coverage path {} is not a file or directory",
                path.display()
            ));
        }
    }
    Ok(files)
}

fn list_files_recursive(path: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let entries = std::fs::read_dir(path).map_err(|err| {
        format!(
            "Failed to read coverage directory {}: {err}",
            path.display()
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|err| {
            format!(
                "Failed to read coverage directory {}: {err}",
                path.display()
            )
        })?;
        let entry_path = entry.path();
        let file_type = entry.file_type().map_err(|err| {
            format!(
                "Failed to read coverage directory {}: {err}",
                path.display()
            )
        })?;
        if file_type.is_dir() {
            files.extend(list_files_recursive(&entry_path)?);
        } else if file_type.is_file() {
            files.push(entry_path);
        }
    }
    Ok(files)
}
