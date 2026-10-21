//! The committed documents must equal a fresh render of their templates, and
//! every number in a template must be either a model placeholder or allowlisted.
use stellar_model::docgen::{lint, parse_allowlist};
use stellar_model::{
    extract_block, render_claims, render_paper, render_paper_block, render_readme, values,
    ALLOWLIST, CLAIMS_DATA, CLAIMS_TEMPLATE, PAPER_TEMPLATE, README_TEMPLATE,
};

const PAPER: &str = include_str!("../../whitepaper/whitepaper.md");
const README: &str = include_str!("../../README.md");
const CLAIMS: &str = include_str!("../../CLAIMS.md");
const FIX: &str = "run `cargo run -q -p stellar-model -- --write` after editing model/templates/";

fn first_diff(a: &str, b: &str) -> String {
    for (i, (x, y)) in a.lines().zip(b.lines()).enumerate() {
        if x != y {
            return format!("line {}:\n  committed: {x}\n  render:    {y}", i + 1);
        }
    }
    format!(
        "line counts differ ({} vs {})",
        a.lines().count(),
        b.lines().count()
    )
}

#[test]
fn paper_equals_fresh_render() {
    let r = render_paper().expect("render");
    assert!(
        PAPER == r,
        "whitepaper.md is stale or hand-edited; {FIX}\n{}",
        first_diff(PAPER, &r)
    );
}

#[test]
fn readme_equals_fresh_render() {
    let r = render_readme().expect("render");
    assert!(
        README == r,
        "README.md is stale or hand-edited; {FIX}\n{}",
        first_diff(README, &r)
    );
}

#[test]
fn claims_equals_fresh_render() {
    let r = render_claims().expect("render");
    assert!(
        CLAIMS == r,
        "CLAIMS.md is stale or hand-edited; {FIX}\n{}",
        first_diff(CLAIMS, &r)
    );
}

#[test]
fn paper_block_equals_model_output() {
    let generated = render_paper_block().expect("reference inputs are valid");
    assert_eq!(extract_block(PAPER), Some(generated.as_str()));
}

#[test]
fn every_template_number_is_generated_or_allowlisted() {
    let allow = parse_allowlist(ALLOWLIST).expect("allowlist parses");
    let vals = values().expect("values");
    let mut problems = Vec::new();
    for (doc, t) in [
        ("paper", PAPER_TEMPLATE),
        ("readme", README_TEMPLATE),
        ("claims", CLAIMS_TEMPLATE),
        ("claims.tsv", CLAIMS_DATA),
    ] {
        let r = lint(doc, t, &allow, &vals);
        problems.extend(
            r.unlisted
                .into_iter()
                .map(|s| format!("NOT GENERATED OR ALLOWLISTED {s}")),
        );
        problems.extend(
            r.unused
                .into_iter()
                .map(|s| format!("STALE ALLOWLIST ROW {s}")),
        );
        problems.extend(
            r.shadows_model
                .into_iter()
                .map(|s| format!("ALLOWLIST SHADOWS A MODEL VALUE {s}")),
        );
    }
    assert!(
        problems.is_empty(),
        "{} problem(s):\n{}",
        problems.len(),
        problems.join("\n")
    );
}

#[test]
fn every_value_key_is_unique() {
    let vals = values().expect("values");
    for (i, v) in vals.iter().enumerate() {
        assert!(
            vals[i + 1..].iter().all(|w| w.key != v.key),
            "duplicate key {}",
            v.key
        );
    }
}

#[test]
fn readme_does_not_demote_the_paper() {
    assert!(!README.to_lowercase().contains("treat it as history"));
}

#[test]
fn readme_has_no_marketing_tagline() {
    assert!(!README.to_lowercase().contains("megacorporation"));
}
