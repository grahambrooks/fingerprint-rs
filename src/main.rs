//! `fingerprint` — winnowing document fingerprints and Jaccard similarity.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use fingerprint_rs::{fingerprint, jaccard, similarity, DEFAULT_K, DEFAULT_T};

#[derive(Parser)]
#[command(
    name = "fingerprint",
    version,
    about = "Winnowing document fingerprints and Jaccard similarity"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Print the Jaccard similarity (0..1) between two files.
    Similarity {
        a: PathBuf,
        b: PathBuf,
        #[arg(short, long, default_value_t = DEFAULT_K, help = "k-gram size (noise threshold)")]
        k: usize,
        #[arg(short, long, default_value_t = DEFAULT_T, help = "winnow window (guarantee threshold, >= k)")]
        t: usize,
    },
    /// Print the winnowing fingerprint (hex hashes) of a file.
    Print {
        file: PathBuf,
        #[arg(short, long, default_value_t = DEFAULT_K)]
        k: usize,
        #[arg(short, long, default_value_t = DEFAULT_T)]
        t: usize,
    },
    /// Report file pairs whose similarity meets a threshold (find duplicated logic).
    Scan {
        /// Files and/or directories to compare (directories are walked).
        paths: Vec<PathBuf>,
        #[arg(short, long, default_value_t = DEFAULT_K)]
        k: usize,
        #[arg(short, long, default_value_t = DEFAULT_T)]
        t: usize,
        #[arg(long, default_value_t = 0.8, help = "minimum similarity to report")]
        threshold: f64,
    },
}

fn read(path: &Path) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("fingerprint: {}: {e}", path.display());
            None
        }
    }
}

/// Expand paths: files pass through, directories are walked (skipping VCS/build
/// and hidden directories).
fn collect(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for p in paths {
        if p.is_dir() {
            walk(p, &mut out);
        } else {
            out.push(p.clone());
        }
    }
    out
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name.starts_with('.') || matches!(name, "target" | "node_modules") {
                continue;
            }
            walk(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn main() -> ExitCode {
    match Cli::parse().cmd {
        Cmd::Similarity { a, b, k, t } => {
            let (Some(sa), Some(sb)) = (read(&a), read(&b)) else {
                return ExitCode::FAILURE;
            };
            println!("{:.4}", similarity(&sa, &sb, k, t));
        }
        Cmd::Print { file, k, t } => {
            let Some(s) = read(&file) else {
                return ExitCode::FAILURE;
            };
            let mut fp: Vec<u32> = fingerprint(&s, k, t).into_iter().collect();
            fp.sort_unstable();
            for h in fp {
                println!("{h:08x}");
            }
        }
        Cmd::Scan {
            paths,
            k,
            t,
            threshold,
        } => {
            let mut fps: Vec<(PathBuf, HashSet<u32>)> = Vec::new();
            for f in collect(&paths) {
                if let Some(s) = read(&f) {
                    let fp = fingerprint(&s, k, t);
                    if !fp.is_empty() {
                        fps.push((f, fp));
                    }
                }
            }
            let mut pairs: Vec<(f64, &PathBuf, &PathBuf)> = Vec::new();
            for i in 0..fps.len() {
                for j in (i + 1)..fps.len() {
                    let sim = jaccard(&fps[i].1, &fps[j].1);
                    if sim >= threshold {
                        pairs.push((sim, &fps[i].0, &fps[j].0));
                    }
                }
            }
            pairs.sort_by(|x, y| y.0.partial_cmp(&x.0).unwrap_or(std::cmp::Ordering::Equal));
            for (sim, a, b) in pairs {
                println!("{:.4}\t{}\t{}", sim, a.display(), b.display());
            }
        }
    }
    ExitCode::SUCCESS
}
