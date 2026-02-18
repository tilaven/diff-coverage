# 0.6.0 - 2026-02-18

Added
- Push README to DockerHub as overview

# 0.5.0 - 2026-02-18

Added
- `--skip-coverage-path` regex option to exclude matching paths from uncovered line calculations and list skipped paths in the summary output
- cargo fmt to GitHub workflow

Changed
- CLI output lines order

# 0.4.1 - 2026-02-15

Fixed
- Bump version in Cargo.toml

# 0.4.0 - 2026-02-15

Changed
- Change entrypoint to cmd to simplify CI usage

# 0.3.2 - 2026-02-11

Fixed 
- dockerhub release

# 0.3.1 - 2026-02-11

Fixed
- Fix Docker releases

# 0.3.0 - 2026-02-11

Changed
- Output binaries as archives in releases
- Dockerhub AND GHCR release

# 0.2.1 - 2026-02-10

Added
- Dockerhub release 

# 0.2.0 - 2026-02-10

Changed
- Output binaries in releases instead of archives with README

# 0.1.6 - 2026-01-30

Fixed
- Invalid version in Cargo.toml

# 0.1.5 - 2026-01-30

Fixed
- Fix coverage of files with the same name in different directories

# 0.1.4 - 2026-01-30

Added
- Add auto publishing to crates.io

# 0.1.2 - 2026-01-29

Fixes
- Fix GitHub Actions release matrix targets - install Cross

# 0.1.1 - 2026-01-29

Fixes
- Fix GitHub Actions release matrix targets (remove stray trailing commas) so macOS builds resolve the correct Rust target.

# 0.1.0 - 2026-01-29

We're excited to ship the very first release of diff-coverage, a developer‑focused CLI that makes coverage on changed code 
fast, reliable, and CI‑friendly. Built for teams who want actionable coverage signals without slow or unreliable pipelines.

What's new
- Blazing‑fast runs: up to ~108× faster mean wall‑time vs Python diff‑cover in larger runs
- Memory‑efficient processing: ~21–28× lower peak RSS memory usage
- Coverage directories supported: point to directories (not just single files) for flexible CI setups
- `--fail-under` support: enforce quality gates directly in your pipeline

Why it matters diff-coverage is designed for modern dev workflows—fast enough for tight inner loops, strict enough for 
CI, and predictable across mono‑repos and multi‑coverage setups.

Thanks for being part of the very first release — more to come!
