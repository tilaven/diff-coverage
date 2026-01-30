# Diff-cover benchmark comparison

Benchmarks were run as one‑time AWS Fargate tasks on a 4‑vCPU, 16‑GB Linux (x86_64) environment.

Parsed GNU time output from .log files under `coverage-download/`. Percentiles use linear interpolation.

## Benchmark 1
- Python logs: 100 files
- Rust logs: 106 files
- Git diff file: 40MB size and 1,808,684 lines long
- Coverage file: 46MB size and 1,115,971 lines long

#### Elapsed wall time (seconds)
| Stat | Python | Rust |
|---|---|---|
| n | 100 | 106 |
| mean | 108.23 | 2.47 |
| median | 108.23 | 2.62 |
| p95 | 109.25 | 2.69 |
| p99 | 109.58 | 3.05 |
| min | 106.82 | 1.72 |
| max | 109.74 | 3.66 |
| stdev | 0.62 | 0.36 |

#### Elapsed wall time comparison (lower is better)
| Stat | Python | Rust | Rust faster (x) | Rust improvement (%) |
|---|---|---|---|---|
| mean | 108.23 | 2.47 | 43.90x | 97.72% |
| median | 108.23 | 2.62 | 41.31x | 97.58% |
| p95 | 109.25 | 2.69 | 40.61x | 97.54% |
| p99 | 109.58 | 3.05 | 35.96x | 97.22% |

#### User CPU time (seconds)
| Stat | Python | Rust |
|---|---|---|
| n | 100 | 106 |
| mean | 105.73 | 2.44 |
| median | 105.74 | 2.59 |
| p95 | 106.51 | 2.65 |
| p99 | 107.15 | 3.04 |
| min | 104.70 | 1.70 |
| max | 107.15 | 3.63 |
| stdev | 0.51 | 0.36 |

#### User CPU time comparison (lower is better)
| Stat | Python | Rust | Rust faster (x) | Rust improvement (%) |
|---|---|---|---|---|
| mean | 105.73 | 2.44 | 43.36x | 97.69% |
| median | 105.74 | 2.59 | 40.83x | 97.55% |
| p95 | 106.51 | 2.65 | 40.19x | 97.51% |
| p99 | 107.15 | 3.04 | 35.30x | 97.17% |

#### System CPU time (seconds)
| Stat | Python | Rust |
|---|---|---|
| n | 100 | 106 |
| mean | 2.49 | 0.02 |
| median | 2.54 | 0.02 |
| p95 | 2.80 | 0.04 |
| p99 | 2.86 | 0.04 |
| min | 1.87 | 0.00 |
| max | 3.04 | 0.04 |
| stdev | 0.25 | 0.01 |

#### System CPU time comparison (lower is better)
| Stat | Python | Rust | Rust faster (x) | Rust improvement (%) |
|---|---|---|---|---|
| mean | 2.49 | 0.02 | 120.36x | 99.17% |
| median | 2.54 | 0.02 | 126.75x | 99.21% |
| p95 | 2.80 | 0.04 | 70.02x | 98.57% |
| p99 | 2.86 | 0.04 | 71.55x | 98.60% |

#### Max RSS (kB)
| Stat | Python | Rust |
|---|---|---|
| n | 100 | 106 |
| mean | 655740 | 23207 |
| median | 655750 | 23208 |
| p95 | 655990 | 23359 |
| p99 | 656036 | 23376 |
| min | 655420 | 23036 |
| max | 656036 | 23384 |
| stdev | 142 | 85 |

#### Max RSS comparison (lower is better)
| Stat | Python | Rust | Rust faster (x) | Rust improvement (%) |
|---|---|---|---|---|
| mean | 655740.28 | 23206.79 | 28.26x | 96.46% |
| median | 655750.00 | 23208.00 | 28.26x | 96.46% |
| p95 | 655989.60 | 23359.00 | 28.08x | 96.44% |
| p99 | 656036.00 | 23375.60 | 28.06x | 96.44% |

## Benchmark 2
- Python logs: 101 files
- Rust logs: 100 files
- Git diff file: 40MB size and 1,808,684 lines long
- 50 coverage files: each 620KB size and 24,116 lines long

#### Elapsed wall time (seconds)
| Stat | Python | Rust |
|---|---|---|
| n | 101 | 100 |
| mean | 227.24 | 2.10 |
| median | 233.86 | 1.91 |
| p95 | 235.86 | 2.65 |
| p99 | 237.65 | 2.69 |
| min | 130.73 | 1.73 |
| max | 240.35 | 4.11 |
| stdev | 24.07 | 0.37 |

#### Elapsed wall time comparison (lower is better)
| Stat | Python | Rust | Rust faster (x) | Rust improvement (%) |
|---|---|---|---|---|
| mean | 227.24 | 2.10 | 108.24x | 99.08% |
| median | 233.86 | 1.91 | 122.44x | 99.18% |
| p95 | 235.86 | 2.65 | 89.00x | 98.88% |
| p99 | 237.65 | 2.69 | 88.20x | 98.87% |

#### User CPU time (seconds)
| Stat | Python | Rust |
|---|---|---|
| n | 101 | 100 |
| mean | 223.05 | 2.08 |
| median | 229.28 | 1.90 |
| p95 | 231.39 | 2.62 |
| p99 | 232.83 | 2.67 |
| min | 128.95 | 1.70 |
| max | 233.89 | 4.09 |
| stdev | 23.72 | 0.37 |

#### User CPU time comparison (lower is better)
| Stat | Python | Rust | Rust faster (x) | Rust improvement (%) |
|---|---|---|---|---|
| mean | 223.05 | 2.08 | 107.33x | 99.07% |
| median | 229.28 | 1.90 | 120.67x | 99.17% |
| p95 | 231.39 | 2.62 | 88.30x | 98.87% |
| p99 | 232.83 | 2.67 | 87.06x | 98.85% |

#### System CPU time (seconds)
| Stat | Python | Rust |
|---|---|---|
| n | 101 | 100 |
| mean | 4.17 | 0.02 |
| median | 4.47 | 0.02 |
| p95 | 5.45 | 0.03 |
| p99 | 6.41 | 0.03 |
| min | 1.77 | 0.00 |
| max | 6.71 | 0.04 |
| stdev | 1.13 | 0.01 |

#### System CPU time comparison (lower is better)
| Stat | Python | Rust | Rust faster (x) | Rust improvement (%) |
|---|---|---|---|---|
| mean | 4.17 | 0.02 | 252.77x | 99.60% |
| median | 4.47 | 0.02 | 223.50x | 99.55% |
| p95 | 5.45 | 0.03 | 181.67x | 99.45% |
| p99 | 6.41 | 0.03 | 212.96x | 99.53% |

#### Max RSS (kB)
| Stat | Python | Rust |
|---|---|---|
| n | 101 | 100 |
| mean | 506218 | 23237 |
| median | 506208 | 23238 |
| p95 | 506396 | 23360 |
| p99 | 506420 | 23368 |
| min | 506044 | 23056 |
| max | 506508 | 23392 |
| stdev | 103 | 76 |

#### Max RSS comparison (lower is better)
| Stat | Python | Rust | Rust faster (x) | Rust improvement (%) |
|---|---|---|---|---|
| mean | 506218.10 | 23237.24 | 21.78x | 95.41% |
| median | 506208.00 | 23238.00 | 21.78x | 95.41% |
| p95 | 506396.00 | 23360.00 | 21.68x | 95.39% |
| p99 | 506420.00 | 23368.24 | 21.67x | 95.39% |

