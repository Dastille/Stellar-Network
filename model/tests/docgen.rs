//! Unit tests of the template renderer and number lint.
use stellar_model::docgen::{
    claims_table, excerpt_body, excerpt_contains, grouped, lint, numeric_tokens, parse_allowlist,
    parse_claims, render, Cmp, DocError, Status, Value,
};

fn vals() -> Vec<Value> {
    vec![
        Value {
            key: "a.kwh",
            text: "5,815".into(),
            derived: true,
        },
        Value {
            key: "a.kwp",
            text: "5".into(),
            derived: false,
        },
    ]
}

#[test]
fn grouping() {
    assert_eq!(grouped(5815.0, 0), "5,815");
    assert_eq!(grouped(4884.6, 2), "4,884.60");
    assert_eq!(grouped(900_000_000.0, 0), "900,000,000");
    assert_eq!(grouped(64.5, 2), "64.50");
    assert_eq!(grouped(-1234.5, 1), "-1,234.5");
}

#[test]
fn renders_and_rejects() {
    assert_eq!(
        render("x {{a.kwh}} y {{block}}", &vals(), &[("block", "B")]).unwrap(),
        "x 5,815 y B"
    );
    assert_eq!(
        render("{{nope}}", &vals(), &[]),
        Err(DocError::UnknownKey("nope".into()))
    );
    assert_eq!(
        render("ab {{a.kwh", &vals(), &[]),
        Err(DocError::Unclosed(3))
    );
}

#[test]
fn tokenizer_skips_identifiers_urls_and_placeholders() {
    let t = "E2b CO2 A277 {{a.kwh}} https://x.org/1163 and 5,815 kWh, $4,884.60. (12)";
    let toks: Vec<String> = numeric_tokens(t).into_iter().map(|n| n.token).collect();
    assert_eq!(toks, vec!["5,815", "4,884.60", "12"]);
}

#[test]
fn lint_catches_hand_typed_model_number() {
    // A model value typed by hand instead of `{{a.kwh}}` is not allowlisted.
    let r = lint("paper", "yields 5,815 kWh", &[], &vals());
    assert_eq!(r.unlisted.len(), 1);
    // Allowlisting it as a quote is caught too.
    let allow = parse_allowlist("paper\t5,815\tyields 5,815\tquote\tX").unwrap();
    let r = lint("paper", "yields 5,815 kWh", &allow, &vals());
    assert_eq!(r.unlisted.len(), 0);
    assert_eq!(r.shadows_model.len(), 1);
}

#[test]
fn lint_whole_token_and_stale_rows() {
    // Context must contain the token as a whole token: `$900` does not match `$900,000`.
    assert!(parse_allowlist("readme\t900\tis $900,000\tquote\tX").is_err());
    assert!(
        parse_allowlist("readme\t900\t $900 \tquote\tX").is_err(),
        "context too short"
    );
    let allow = parse_allowlist(
        "readme\t900\tCA$75 is $900 a year\tquote\tX\nreadme\t7\tnowhere 7\tauthor\tY",
    )
    .unwrap();
    let r = lint("readme", "CA$75 is $900 a year", &allow, &[]);
    assert!(r.unlisted.iter().all(|s| !s.contains("`900`")));
    assert_eq!(r.unused.len(), 1);
}

fn toks(t: &str) -> Vec<String> {
    numeric_tokens(t).into_iter().map(|n| n.token).collect()
}

#[test]
fn tokenizer_catches_prefixed_and_suffixed_numbers() {
    // Currency prefixes and unit/scale suffixes do not hide a number.
    assert_eq!(toks("cost $227.0B and $999B"), vec!["227.0", "999"]);
    assert_eq!(
        toks("45pct, 45%, 3pp, 1.8× and 1.8x"),
        vec!["45", "45", "3", "1.8", "1.8"]
    );
    assert_eq!(
        toks("5,815kWh or 50lb, 50 lb/week"),
        vec!["5,815", "50", "50"]
    );
    // Ranges split on the en dash; both ends are tokens.
    assert_eq!(toks("45–90 and 4.5–9"), vec!["45", "90", "4.5", "9"]);
    // HTML comments and reference lines are scanned like any other text.
    assert_eq!(toks("<!-- hidden 5,915 -->"), vec!["5,915"]);
    assert_eq!(
        toks("- [CMHC-2025] CMHC, June 19, 2025, p. 12."),
        vec!["2025", "19", "2025", "12"]
    );
    // Only letter-prefixed identifiers are skipped.
    assert_eq!(toks("R14 v2 FY2024 E2b CO2 A277 x86"), Vec::<String>::new());
}

#[test]
fn lint_catches_typed_suffixed_number_in_template() {
    let r = lint("paper", "Savings reach $227.0B a year", &[], &vals());
    assert_eq!(r.unlisted.len(), 1, "{r:?}");
    assert!(r.unlisted[0].contains("`227.0`"));
}

#[test]
fn allowlist_row_locates_one_occurrence_only() {
    // The row covers the `2025` in the reference date, not a second 2025
    // typed elsewhere on the same line.
    let allow = parse_allowlist("paper\t2025\tCMHC, June 19, 2025\tdate\tref").unwrap();
    let line = "CMHC, June 19, 2025; also 2025 units";
    let r = lint("paper", line, &allow, &[]);
    let missing: Vec<&String> = r.unlisted.iter().filter(|s| s.contains("`2025`")).collect();
    assert_eq!(missing.len(), 1, "{r:?}");
}

#[test]
fn comparison_words_follow_the_values() {
    assert_eq!(Cmp::of(4949.10, 4884.60, 2).exceeds(), "exceeds");
    assert_eq!(Cmp::of(4949.10, 4884.60, 2).exceed(), "exceed");
    assert_eq!(Cmp::of(3884.60, 4884.60, 2).exceeds(), "is below");
    assert_eq!(Cmp::of(3884.60, 4884.60, 2).exceed(), "are below");
    assert_eq!(Cmp::of(1.004, 1.0, 2), Cmp::Equal);
}

#[test]
fn claims_rows_render_and_reject_bad_status() {
    let tsv = "# comment\nyield {{a.kwh}}\t`model` E2\tderived; upper bound\t-\t-\nq\tsrc\tverified\ta.txt + b.txt\t{{a.kwh}} kWh || x\n";
    let c = parse_claims(tsv, &vals()).unwrap();
    assert_eq!(c.len(), 2);
    assert_eq!(c[0].claim, "yield 5,815");
    assert_eq!(c[0].status, Status::Derived);
    assert_eq!(c[0].note, "upper bound");
    assert_eq!(c[1].evidence, vec!["a.txt", "b.txt"]);
    assert_eq!(c[1].needles, vec!["5,815 kWh", "x"]);
    assert!(claims_table(&c).contains("| 1 | yield 5,815 | `model` E2 | derived (upper bound) |"));
    assert!(parse_claims("q\ts\tprobably\t-\t-", &vals()).is_err());
    assert!(parse_claims("q\ts", &vals()).is_err());
}

#[test]
fn excerpts_need_a_header_and_match_modulo_whitespace() {
    let ok = "# Source: S\n# URL: u\n# Accessed: d\n# Method: m\n---\nTier 1 |   12.0  |\n";
    let body = excerpt_body(ok).unwrap();
    assert!(excerpt_contains(body, "Tier 1 | 12.0 |"));
    assert!(!excerpt_contains(body, "Tier 1 | 13.0 |"));
    assert!(excerpt_body("# Source: S\n---\nx").is_err());
}
