# ⚡🧠️ diff-coverage

Diff-cover, supercharged in Rust 🦀. Get instant, actionable coverage insights at ⚡ blazing speed ⚡ with
a 🧠️ tiny memory footprint 🧠️. Built for modern CI and massive repos, it turns slow coverage checks into
fast, reliable feedback you can trust.

## Usage

```shell
diff-coverage coverage.xml --diff-file diff.diff

# Multiple coverage inputs (repeat or comma‑separated)
diff-coverage coverage1.xml coverage2.xml --diff-file diff.diff

# Multiple coverage inputs in a directory
diff-coverage ./coverage/ --diff-file diff.diff

# Fail the build if diff coverage drops below a threshold
diff-coverage ./coverage/ coverage.xml --diff-file diff.diff --fail-under 80

# Output to CI formats
diff-coverage coverage.xml --diff-file diff.diff --output gitlab=diff-cover.json
diff-coverage coverage.xml --diff-file diff.diff --output json=diff-cover.json --output summary

# Skip external packages from uncovered line calculations (regex supported)
diff-coverage coverage.xml --diff-file diff.diff --skip-coverage-path '^pkg/mongodb'
```

## Options

- <a id="diff-file"></a>`--diff-file <PATH>`: diff to analyze
- <a id="fail-under"></a>`--fail-under <PERCENT>`: minimum acceptable diff coverage
- <a id="missing-coverage"></a>`--missing-coverage <MODE>`: how to handle files missing from coverage (uncovered or ignore, default: ignore)
- <a id="skip-coverage-path"></a>`--skip-coverage-path <REGEX>`: regex of file paths to skip from uncovered line calculations (repeatable or comma-separated)
- <a id="output"></a>`--output <FORMAT=PATH>`: output target(s), repeatable or comma‑separated Formats: cli, summary, gitlab, json (note: cli and summary don’t take a path)
- <a id="help"></a>`-h, --help`: show help
- <a id="version"></a>`-V, --version`: show version

## Performance highlights

### Benchmark 1

- Git diff file: 40MB size and 1,808,684 lines long
- Coverage file: 46MB size and 1,115,971 lines long

| Metric | Rust | Python | Improvement (Python / Rust) |
|---|---:|---:|---:|
| Mean wall time | 2.47s | 108.23s | 43.90× |
| Max wall time | 3.66s | 109.74s | 29.98× |
| Mean peak RSS | 22.66 MB | 640.37 MB | 28.26× |
| Max peak RSS | 22.84 MB | 640.66 MB | 28.05× |

### Benchmark 2

- Git diff file: 40MB size and 1,808,684 lines long
- 50 coverage files: each 620KB size and 24,116 lines long

| Metric | Rust | Python | Improvement (Python / Rust) |
|---|---:|---:|---:|
| Mean wall time | 2.10s | 227.24s | 108.24× |
| Max wall time | 4.11s | 240.35s | 58.48× |
| Mean peak RSS | 22.69 MB | 494.35 MB | 21.78× |
| Max peak RSS | 22.84 MB | 494.64 MB | 21.65× |

*Peak RSS shown in MB (kB ÷ 1024).*  
[Full benchmark results](docs/benchmark_results.md)

## Installation

### Prebuilt binaries

Download the prebuilt archives from the [GitHub Releases page](https://github.com/tilaven/diff-coverage/releases).

### Cargo install

```bash
cargo install diff-coverage
```

### Docker

```bash
docker run --rm -v "$PWD":/work -w /work tilaven/diff-coverage:<version>-alpine \
  coverage.xml --diff-file diff.diff
```

Debian-based image (e.g. trixie) is available via versioned tags:

```bash
docker run --rm -v "$PWD":/work -w /work tilaven/diff-coverage:<version>-debian-trixie \
  coverage.xml --diff-file diff.diff
```

Minimal scratch image is available via versioned tags:

```bash
docker run --rm -v "$PWD":/work -w /work tilaven/diff-coverage:<version>-scratch \
  coverage.xml --diff-file diff.diff
```

## Build

```bash
cargo build
```

## Run

```bash
cargo run
```

## Test

```bash
cargo test
```

## Release (GitHub Actions)

Push a version tag and GitHub Actions will run tests and attach executables to the GitHub Release:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Download the prebuilt archives from the GitHub Releases page.
