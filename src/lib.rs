//! Winnowing document fingerprinting and Jaccard similarity.
//!
//! A faithful Rust port of the `github.com/grahambrooks/fingerprint` Go library,
//! implementing the algorithm from *Winnowing: Local Algorithms for Document
//! Fingerprinting* (Schleimer, Wilkerson, Aiken). The pipeline is:
//!
//! ```text
//! clean text -> k-grams -> FNV-1a hashes -> winnow -> set of selected hashes
//! ```
//!
//! Two documents are compared with the Jaccard index over their fingerprint sets.
//! Small edits perturb only a few k-grams, so the fingerprint (and similarity) is
//! robust — the property that makes it useful for spotting duplicated logic.

use std::collections::HashSet;

/// Noise threshold `k`: the k-gram length. The default when unspecified/invalid.
pub const DEFAULT_K: usize = 4;
/// Guarantee threshold `t`: the winnowing window size. Must be `>= k`.
pub const DEFAULT_T: usize = 4;

/// Keep only Unicode letters, lowercased — matches the Go `text.Clean`.
pub fn clean(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphabetic())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// FNV-1a 32-bit hash (matches Go's `hash/fnv.New32a`).
pub fn fnv1a_32(bytes: &[u8]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for &b in bytes {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// Hash every length-`k` byte window of `input` with FNV-1a.
pub fn kgram_hashes(k: usize, input: &str) -> Vec<u32> {
    let bytes = input.as_bytes();
    if k == 0 || bytes.len() < k {
        return Vec::new();
    }
    (0..=bytes.len() - k)
        .map(|i| fnv1a_32(&bytes[i..i + k]))
        .collect()
}

/// Winnow: in each window of `g` consecutive hashes select the rightmost minimum
/// value, returning the selected hashes. `<=` in the comparison makes ties resolve
/// to the rightmost element, exactly as the reference implementation does.
pub fn winnow(g: usize, hashes: &[u32]) -> Vec<u32> {
    if g == 0 || hashes.len() < g {
        return Vec::new();
    }
    (0..=hashes.len() - g)
        .map(|i| {
            let mut min = u32::MAX;
            for &v in &hashes[i..i + g] {
                if v <= min {
                    min = v;
                }
            }
            min
        })
        .collect()
}

/// Resolve `(k, t)` the way the Go `Options.VerifyOrDefault` does: valid iff
/// `0 < k <= t`, otherwise both fall back to the defaults.
fn resolve(k: usize, t: usize) -> (usize, usize) {
    if k > 0 && k <= t {
        (k, t)
    } else {
        (DEFAULT_K, DEFAULT_T)
    }
}

/// Fingerprint of `text`: the set of winnowed FNV-1a k-gram hashes.
pub fn fingerprint(text: &str, k: usize, t: usize) -> HashSet<u32> {
    let (k, t) = resolve(k, t);
    let hashes = kgram_hashes(k, &clean(text));
    winnow(t, &hashes).into_iter().collect()
}

/// Jaccard index between two fingerprint sets: `|A ∩ B| / |A ∪ B|`.
pub fn jaccard(a: &HashSet<u32>, b: &HashSet<u32>) -> f64 {
    let union = a.union(b).count();
    if union == 0 {
        return 0.0;
    }
    a.intersection(b).count() as f64 / union as f64
}

/// Jaccard similarity of two strings under the given thresholds.
pub fn similarity(s1: &str, s2: &str, k: usize, t: usize) -> f64 {
    jaccard(&fingerprint(s1, k, t), &fingerprint(s2, k, t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_keeps_only_lowercased_letters() {
        assert_eq!(clean("Hello, World! 123"), "helloworld");
    }

    #[test]
    fn fnv1a_matches_reference_vector() {
        // The Go test pins KGramHash(5, "adorunrunrunadorunrun"); the first
        // 5-gram is "adoru" and must hash to 0xf765d270 under FNV-1a/32.
        assert_eq!(fnv1a_32(b"adoru"), 0xf765_d270);
    }

    #[test]
    fn empty_text_has_empty_fingerprint() {
        assert!(fingerprint("", 4, 4).is_empty());
    }

    #[test]
    fn identical_text_is_perfectly_similar() {
        let s = "the quick brown fox jumps over the lazy dog";
        assert_eq!(similarity(s, s, DEFAULT_K, DEFAULT_T), 1.0);
    }

    #[test]
    fn small_edit_stays_highly_similar() {
        let a = "the quick brown fox jumps over the lazy dog";
        let b = "the quick brown fox jumped over the lazy dog";
        assert!(similarity(a, b, DEFAULT_K, DEFAULT_T) > 0.5);
    }

    #[test]
    fn unrelated_text_is_dissimilar() {
        let a = "the quick brown fox jumps over the lazy dog";
        let b = "completely different content sharing little structure";
        assert!(similarity(a, b, DEFAULT_K, DEFAULT_T) < 0.25);
    }

    #[test]
    fn invalid_options_fall_back_to_defaults() {
        // t < k is invalid -> defaults (4,4); should behave like the default call.
        let s = "adorunrunrunadorunrun";
        assert_eq!(
            fingerprint(s, 9, 2),
            fingerprint(s, DEFAULT_K, DEFAULT_T)
        );
    }
}
