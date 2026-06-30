//! Dashboard footprint check: enforce the gzipped transfer budget per capability tier.
//!
//! "Best-looking on cheap hardware" means fast and tiny, so `.docs/LOCAL_DASHBOARDS.md`
//! sets a hard transfer budget for each tier. This sums the gzipped size of what a browser
//! actually fetches on first load and fails if it grows past the tier's budget, so the size
//! is visible on every run and a regression is caught in CI rather than in the field.
//!
//! - Tiers A and B (the full localized app) load the shell, the styles, every script, and one
//!   locale, against the full-tier budget.
//! - Tier C (the minimal floor) loads only the self-contained `lite.html`, against the much
//!   smaller floor budget.
//!
//! `cargo xtask dashboard footprint` checks every tier; `--tier <a|b|c>` checks just one.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use flate2::write::GzEncoder;
use flate2::Compression;

/// The full-tier budget (A and B): the rich app plus one locale, gzipped.
const TIER_AB_BUDGET: usize = 150 * 1024;

/// The floor-tier budget (C): the single self-contained page, gzipped.
const TIER_C_BUDGET: usize = 50 * 1024;

/// A capability tier, which fixes both the page-load file set and the budget.
#[derive(Clone, Copy)]
enum Tier {
    A,
    B,
    C,
}

impl Tier {
    fn key(self) -> &'static str {
        match self {
            Tier::A => "a",
            Tier::B => "b",
            Tier::C => "c",
        }
    }

    fn budget(self) -> usize {
        match self {
            Tier::C => TIER_C_BUDGET,
            Tier::A | Tier::B => TIER_AB_BUDGET,
        }
    }
}

/// Run the `dashboard footprint` task: report and enforce the per-tier gzipped budget.
///
/// # Arguments
///
/// * `args` - an optional `--tier <a|b|c>` to check a single tier; with none, every tier is
///   checked.
///
/// # Returns
///
/// Success when every checked tier is within budget, otherwise a failure.
pub fn run(args: &[String]) -> ExitCode {
    let tiers = match parse_tiers(args) {
        Ok(tiers) => tiers,
        Err(message) => {
            eprintln!("xtask dashboard footprint: {message}");
            return ExitCode::FAILURE;
        }
    };

    let web = web_dir();
    let mut ok = true;
    for tier in tiers {
        if let Err(message) = check(&web, tier) {
            eprintln!("xtask dashboard footprint: {message}");
            ok = false;
        }
    }

    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

// Reads an optional `--tier <a|b|c>`; with none, returns every tier.
fn parse_tiers(args: &[String]) -> Result<Vec<Tier>, String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--tier" {
            let value = iter.next().ok_or("--tier needs a value: a, b, or c")?;
            return Ok(vec![tier_from(value)?]);
        }
    }
    Ok(vec![Tier::A, Tier::B, Tier::C])
}

fn tier_from(value: &str) -> Result<Tier, String> {
    match value.to_ascii_lowercase().as_str() {
        "a" => Ok(Tier::A),
        "b" => Ok(Tier::B),
        "c" => Ok(Tier::C),
        other => Err(format!("unknown tier {other:?}; expected a, b, or c")),
    }
}

fn web_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repo root is two levels above the xtask crate")
        .join("crates/pamoja-dashboard/web")
}

// The files a browser fetches on first load for the given tier. The full tiers take the
// shell, styles, every script, and one locale (the budget is "including one locale"); the
// floor tier takes only the self-contained page. `lite.html` is the floor page, so it is
// excluded from the full tiers where it is never loaded.
fn page_load_files(web: &Path, tier: Tier) -> Result<Vec<PathBuf>, String> {
    if let Tier::C = tier {
        return Ok(vec![web.join("lite.html")]);
    }
    let mut all = Vec::new();
    collect(web, &mut all).map_err(|e| format!("walking {}: {e}", web.display()))?;
    let mut chosen: Vec<PathBuf> = all
        .into_iter()
        .filter(|p| {
            let is_web_asset = matches!(
                p.extension().and_then(|e| e.to_str()),
                Some("html") | Some("css") | Some("js")
            );
            let is_floor_page = p.file_name().and_then(|n| n.to_str()) == Some("lite.html");
            is_web_asset && !is_floor_page
        })
        .collect();
    chosen.push(web.join("app/i18n/en.json"));
    chosen.sort();
    Ok(chosen)
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

fn check(web: &Path, tier: Tier) -> Result<(), String> {
    let budget = tier.budget();
    let mut rows = Vec::new();
    let mut total = 0usize;
    for path in page_load_files(web, tier)? {
        let bytes = fs::read(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
        let size = gzipped_len(&bytes);
        total += size;
        let rel = path
            .strip_prefix(web)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        rows.push((size, rel));
    }
    rows.sort_by_key(|row| std::cmp::Reverse(row.0));
    println!("tier {}:", tier.key());
    for (size, rel) in &rows {
        println!("  {size:>6}  {rel}");
    }
    println!("  total {total} bytes gzipped, budget {budget}\n");
    if total > budget {
        Err(format!(
            "tier {} page-load bundle is {total} bytes gzipped, over the {budget} budget by {}",
            tier.key(),
            total - budget
        ))
    } else {
        Ok(())
    }
}

fn gzipped_len(bytes: &[u8]) -> usize {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(bytes).expect("gzip write");
    encoder.finish().expect("gzip finish").len()
}
