# fingerprint-rs

Winnowing document fingerprinting and Jaccard similarity, as a Rust library and a
`fingerprint` CLI. A port of [`grahambrooks/fingerprint`](https://github.com/grahambrooks/fingerprint)
(Go), implementing the algorithm from
[*Winnowing: Local Algorithms for Document Fingerprinting*](http://theory.stanford.edu/~aiken/publications/papers/sigmod03.pdf).

Small edits perturb only a few k-grams, so the fingerprint — and the similarity
score — is robust. That makes it useful for spotting **duplicated logic** across a
codebase (connascence of algorithm).

## Install

```shell
brew tap grahambrooks/fingerprint-rs https://github.com/grahambrooks/fingerprint-rs
brew install grahambrooks/fingerprint-rs/fingerprint
```

Or from source: `cargo install --git https://github.com/grahambrooks/fingerprint-rs --locked`.

## CLI

```shell
# Jaccard similarity (0..1) between two files
fingerprint similarity a.rs b.rs

# The winnowing fingerprint (hex hashes) of a file
fingerprint print src/main.rs

# Find similar file pairs (duplicated logic) across files/dirs
fingerprint scan src/ --threshold 0.8
```

Options: `-k` k-gram size (noise threshold, default 4), `-t` winnow window
(guarantee threshold, `>= k`, default 4).

## Library

```rust
use fingerprint_rs::{similarity, DEFAULT_K, DEFAULT_T};

let score = similarity(text_a, text_b, DEFAULT_K, DEFAULT_T); // 0.0..=1.0
```

Also exposes `clean`, `fnv1a_32`, `kgram_hashes`, `winnow`, `fingerprint`, and
`jaccard` for granular control.

## License

MIT
