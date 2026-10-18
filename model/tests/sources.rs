//! Every claim marked verified must be backed by a committed verbatim excerpt
//! of its source (`model/sources/*.txt`, with URL, access time and method),
//! and every value listed for that claim, rendered from the model's own
//! constants, must appear in the excerpt. Changing a sourced constant without
//! new evidence therefore fails `cargo test`.
use std::path::PathBuf;
use stellar_model::claims;
use stellar_model::docgen::{excerpt_body, excerpt_contains, Status};

fn sources_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sources")
}

fn read_excerpt(name: &str) -> Result<String, String> {
    let path = sources_dir().join(name);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    excerpt_body(&text)
        .map(str::to_string)
        .map_err(|e| format!("{name}: {e}"))
}

#[test]
fn verified_claims_have_excerpts_containing_their_values() {
    let mut problems = Vec::new();
    for c in claims().expect("claims render") {
        let needs_evidence = matches!(c.status, Status::Verified | Status::PartlyVerified);
        if needs_evidence && (c.evidence.is_empty() || c.needles.is_empty()) {
            problems.push(format!(
                "claim {} is {} but has no excerpt or no values to check",
                c.id,
                c.status.word()
            ));
            continue;
        }
        if c.evidence.is_empty() {
            if !c.needles.is_empty() {
                problems.push(format!("claim {} lists values but no excerpt", c.id));
            }
            continue;
        }
        let mut body = String::new();
        for e in &c.evidence {
            match read_excerpt(e) {
                Ok(b) => body.push_str(&b),
                Err(err) => problems.push(format!("claim {}: {err}", c.id)),
            }
        }
        for n in &c.needles {
            if !excerpt_contains(&body, n) {
                problems.push(format!(
                    "claim {}: `{n}` not found in {}",
                    c.id,
                    c.evidence.join(" + ")
                ));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "{} problem(s):\n{}",
        problems.len(),
        problems.join("\n")
    );
}

#[test]
fn derived_claims_cite_the_model() {
    for c in claims().expect("claims render") {
        if c.status == Status::Derived {
            assert!(
                c.source.contains("`model`"),
                "derived claim {} does not cite `model`",
                c.id
            );
        }
    }
}

#[test]
fn every_committed_excerpt_is_used_and_well_formed() {
    let used: Vec<String> = claims()
        .expect("claims render")
        .into_iter()
        .flat_map(|c| c.evidence)
        .collect();
    for entry in std::fs::read_dir(sources_dir()).expect("sources dir") {
        let name = entry
            .expect("entry")
            .file_name()
            .to_string_lossy()
            .into_owned();
        assert!(read_excerpt(&name).is_ok(), "{name} is malformed");
        assert!(used.contains(&name), "{name} is not cited by any claim");
    }
}
