# 0.1.2 - 2026-01-29

Fixes
- Fix GitHub Actions release matrix targets - install Cross
`- `
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
