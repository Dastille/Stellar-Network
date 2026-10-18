//! Document generation: `{{key}}` templates rendered from model values, and a
//! lint that every other number in a template is on an explicit allowlist.
//!
//! The committed paper, `README.md` and `CLAIMS.md` are *outputs*. Their sources
//! live in `model/templates/`. A test asserts each committed file equals a fresh
//! render, so no model-derived number can be edited by hand without failing
//! `cargo test`.

use std::fmt;

/// A named value that templates may reference as `{{key}}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value {
    /// Placeholder key, e.g. `e2.annual_kwh`.
    pub key: &'static str,
    /// Rendered text inserted in place of the placeholder.
    pub text: String,
    /// `true` if the value is computed by the model (not a sourced input or
    /// scenario parameter). Derived values may never appear on the allowlist.
    pub derived: bool,
}

/// Errors raised while rendering or linting a template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocError {
    /// A `{{key}}` names no model value.
    UnknownKey(String),
    /// A `{{` has no matching `}}`; the payload is the byte offset.
    Unclosed(usize),
    /// An allowlist row is malformed; the payload describes the row.
    BadAllowlist(String),
    /// A claims row is malformed; the payload describes the row.
    BadClaim(String),
}

impl fmt::Display for DocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocError::UnknownKey(k) => write!(f, "template references unknown key `{k}`"),
            DocError::Unclosed(at) => write!(f, "unclosed `{{{{` at byte {at}"),
            DocError::BadAllowlist(row) => write!(f, "malformed allowlist row: {row}"),
            DocError::BadClaim(row) => write!(f, "malformed claims row: {row}"),
        }
    }
}

impl std::error::Error for DocError {}

/// Format `x` with `dp` decimals and comma thousands separators.
#[must_use]
pub fn grouped(x: f64, dp: usize) -> String {
    let s = format!("{:.*}", dp, x.abs());
    let (int, frac) = match s.find('.') {
        Some(i) => (&s[..i], &s[i..]),
        None => (s.as_str(), ""),
    };
    let mut out = String::new();
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    let sign = if x < 0.0 && s.chars().any(|c| c.is_ascii_digit() && c != '0') {
        "-"
    } else {
        ""
    };
    format!("{sign}{out}{frac}")
}

/// Replace every `{{key}}` in `template` with the matching value's text, or
/// with the matching named block (e.g. `{{block}}`, `{{claims_table}}`).
///
/// # Errors
/// [`DocError::UnknownKey`] or [`DocError::Unclosed`].
pub fn render(
    template: &str,
    values: &[Value],
    blocks: &[(&str, &str)],
) -> Result<String, DocError> {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    let mut offset = 0;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after.find("}}").ok_or(DocError::Unclosed(offset + start))?;
        let key = after[..end].trim();
        if let Some((_, b)) = blocks.iter().find(|(k, _)| *k == key) {
            out.push_str(b);
        } else {
            let v = values
                .iter()
                .find(|v| v.key == key)
                .ok_or_else(|| DocError::UnknownKey(key.to_string()))?;
            out.push_str(&v.text);
        }
        let consumed = start + 2 + end + 2;
        offset += consumed;
        rest = &rest[consumed..];
    }
    out.push_str(rest);
    Ok(out)
}

/// The result of comparing two model quantities at a printed precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmp {
    /// The first quantity is larger.
    Above,
    /// Both round to the same printed value.
    Equal,
    /// The first quantity is smaller.
    Below,
}

impl Cmp {
    /// Compare `a` with `b` after rounding both to `dp` decimals, so the word
    /// always agrees with the printed numbers.
    #[must_use]
    pub fn of(a: f64, b: f64, dp: usize) -> Cmp {
        let (x, y) = (format!("{a:.dp$}"), format!("{b:.dp$}"));
        let (x, y): (f64, f64) = (x.parse().unwrap_or(a), y.parse().unwrap_or(b));
        if x > y {
            Cmp::Above
        } else if x < y {
            Cmp::Below
        } else {
            Cmp::Equal
        }
    }

    /// Verb for a singular subject: `exceeds`, `equals` or `is below`.
    #[must_use]
    pub fn exceeds(self) -> &'static str {
        match self {
            Cmp::Above => "exceeds",
            Cmp::Equal => "equals",
            Cmp::Below => "is below",
        }
    }

    /// Verb for a plural subject: `exceed`, `equal` or `are below`.
    #[must_use]
    pub fn exceed(self) -> &'static str {
        match self {
            Cmp::Above => "exceed",
            Cmp::Equal => "equal",
            Cmp::Below => "are below",
        }
    }
}

/// One numeric token found in a template line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumToken {
    /// 1-based line number in the template.
    pub line: usize,
    /// The token, digits with internal `,` or `.` (e.g. `4,884.60`).
    pub token: String,
    /// Byte offset of the token in `text`.
    pub at: usize,
    /// The template line with placeholders and URLs blanked.
    pub text: String,
}

fn blank_spans(line: &str) -> String {
    // Blank `{{...}}` placeholders and URLs so they are not linted.
    let mut s = line.to_string();
    while let Some(a) = s.find("{{") {
        let Some(b) = s[a..].find("}}") else { break };
        s.replace_range(a..a + b + 2, " ");
    }
    for scheme in ["https://", "http://"] {
        while let Some(a) = s.find(scheme) {
            let tail = &s[a..];
            let len = tail
                .find(|c: char| c.is_whitespace() || c == ')' || c == '>' || c == '|')
                .unwrap_or(tail.len());
            s.replace_range(a..a + len, " ");
        }
    }
    s
}

/// All numeric tokens of `template`, outside placeholders and URLs.
/// A token starts at an ASCII digit that is not preceded by a letter, digit or
/// `_` (so identifiers such as `E2b`, `CO2` or `A277` are not tokens) and
/// continues over digits and over `,`, `.` or ASCII `-` when followed by a
/// digit (so `2025-11-01` and `1200-2021` are single tokens; ranges in this
/// project use an en dash, which splits).
#[must_use]
pub fn numeric_tokens(template: &str) -> Vec<NumToken> {
    let mut found = Vec::new();
    for (i, raw) in template.lines().enumerate() {
        let text = blank_spans(raw);
        let chars: Vec<char> = text.chars().collect();
        let mut j = 0;
        while j < chars.len() {
            let starts = chars[j].is_ascii_digit()
                && (j == 0 || !(chars[j - 1].is_alphanumeric() || chars[j - 1] == '_'));
            if starts {
                let mut k = j;
                while k < chars.len()
                    && (chars[k].is_ascii_digit()
                        || ((chars[k] == ',' || chars[k] == '.' || chars[k] == '-')
                            && k + 1 < chars.len()
                            && chars[k + 1].is_ascii_digit()))
                {
                    k += 1;
                }
                let at = chars[..j].iter().map(|c| c.len_utf8()).sum();
                found.push(NumToken {
                    line: i + 1,
                    token: chars[j..k].iter().collect(),
                    at,
                    text: text.clone(),
                });
                j = k;
            } else if chars[j].is_ascii_digit() {
                // Skip the rest of an identifier such as `E2b` or `A277`.
                while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '.') {
                    j += 1;
                }
            } else {
                j += 1;
            }
        }
    }
    found
}

/// Kinds of allowlisted (non-model) numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A direct quote of a number from a cited source; `source` names it.
    Quote,
    /// Ashlynn's own figure from an earlier version, quoted as hers.
    Author,
    /// An identifier: standard, regulation, section, SKU, version, list item.
    Id,
    /// A calendar date, year or time.
    Date,
    /// An exact unit or calendar constant used in a stated formula
    /// (e.g. 365 days per year, 12 inches per foot).
    Unit,
}

/// One allowlist row: `doc<TAB>token<TAB>context<TAB>kind<TAB>source`, where
/// kind is `quote`, `author`, `id`, `date` or `unit`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allow {
    /// Template name the row applies to (e.g. `paper`, `readme`, `claims`).
    pub doc: String,
    /// The exact numeric token.
    pub token: String,
    /// A substring of the line that locates the token.
    pub context: String,
    /// Why the number is not generated.
    pub kind: Kind,
    /// Source reference or reason.
    pub source: String,
}

fn whole_token_in(hay: &str, token: &str) -> bool {
    let mut from = 0;
    while let Some(p) = hay[from..].find(token) {
        let a = from + p;
        let b = a + token.len();
        let before = hay[..a].chars().next_back();
        let after = hay[b..].chars().next();
        let mut after2 = hay[b..].chars();
        after2.next();
        let next2 = after2.next();
        let left_ok = before.map_or(true, |c| !(c.is_alphanumeric() || c == '_'));
        let right_ok = match after {
            Some(c) if c.is_ascii_digit() => false,
            Some(',' | '.' | '-') => !next2.is_some_and(|d| d.is_ascii_digit()),
            _ => true,
        };
        if left_ok && right_ok {
            return true;
        }
        from = a + 1;
    }
    false
}

/// Parse the allowlist TSV. Lines starting with `#` and blank lines are skipped.
///
/// # Errors
/// [`DocError::BadAllowlist`] if a row has the wrong shape, an unknown kind,
/// an empty source, or a context that does not contain its token as a whole
/// token plus at least 6 more characters.
pub fn parse_allowlist(tsv: &str) -> Result<Vec<Allow>, DocError> {
    let mut rows = Vec::new();
    for line in tsv.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() != 5 || f[4].trim().is_empty() {
            return Err(DocError::BadAllowlist(line.to_string()));
        }
        let kind = match f[3] {
            "quote" => Kind::Quote,
            "author" => Kind::Author,
            "id" => Kind::Id,
            "date" => Kind::Date,
            "unit" => Kind::Unit,
            _ => return Err(DocError::BadAllowlist(line.to_string())),
        };
        // The context must locate the token: it contains the token as a whole
        // token and at least 6 further characters.
        if !whole_token_in(f[2], f[1]) || f[2].chars().count() < f[1].chars().count() + 6 {
            return Err(DocError::BadAllowlist(line.to_string()));
        }
        rows.push(Allow {
            doc: f[0].to_string(),
            token: f[1].to_string(),
            context: f[2].to_string(),
            kind,
            source: f[4].to_string(),
        });
    }
    Ok(rows)
}

/// Problems found by [`lint`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LintReport {
    /// Numbers in the template that are neither placeholders nor allowlisted.
    pub unlisted: Vec<String>,
    /// Allowlist rows that match nothing (stale).
    pub unused: Vec<String>,
    /// Allowlist rows whose token equals a model-derived value.
    pub shadows_model: Vec<String>,
}

impl LintReport {
    /// `true` if no problem was found.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.unlisted.is_empty() && self.unused.is_empty() && self.shadows_model.is_empty()
    }
}

/// `true` if some occurrence of `context` in `text` spans the token at byte
/// `at` (length `len`): the allowlist row locates *this* token, not merely a
/// line that happens to contain the same digits elsewhere.
fn context_covers(text: &str, context: &str, at: usize, len: usize) -> bool {
    let mut from = 0;
    while let Some(p) = text[from..].find(context) {
        let a = from + p;
        if a <= at && at + len <= a + context.len() {
            return true;
        }
        from = a + text[a..].chars().next().map_or(1, char::len_utf8);
    }
    false
}

fn digits_only(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect()
}

/// Check every numeric token of `template` (named `doc`) against `allow`, and
/// check that no quote/author row restates a derived model value.
#[must_use]
pub fn lint(doc: &str, template: &str, allow: &[Allow], values: &[Value]) -> LintReport {
    let rows: Vec<&Allow> = allow.iter().filter(|a| a.doc == doc).collect();
    let mut used = vec![false; rows.len()];
    let mut report = LintReport::default();
    for t in numeric_tokens(template) {
        let mut covered = false;
        for (i, a) in rows.iter().enumerate() {
            if a.token == t.token && context_covers(&t.text, &a.context, t.at, t.token.len()) {
                used[i] = true;
                covered = true;
            }
        }
        if !covered {
            report.unlisted.push(format!(
                "{doc}:{}: `{}` at byte {} in: {}",
                t.line, t.token, t.at, t.text
            ));
        }
    }
    let derived: Vec<String> = values
        .iter()
        .filter(|v| v.derived)
        .map(|v| digits_only(&v.text))
        .collect();
    for (i, a) in rows.iter().enumerate() {
        if !used[i] {
            report
                .unused
                .push(format!("{doc}: `{}` / `{}`", a.token, a.context));
        }
        if matches!(a.kind, Kind::Quote | Kind::Author) && derived.contains(&digits_only(&a.token))
        {
            report
                .shadows_model
                .push(format!("{doc}: `{}` / `{}`", a.token, a.context));
        }
    }
    report
}

/// The status of one claim in `CLAIMS.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Checked against the cited primary source; an excerpt is committed.
    Verified,
    /// Part of the claim is in the source, part is not.
    PartlyVerified,
    /// Computed by `model/` from sourced inputs.
    Derived,
    /// Read through a reproduction or secondary copy only.
    Secondary,
    /// No source found.
    Unverified,
    /// The evidence disagrees with the claim.
    Contradicted,
    /// Removed from the documents.
    Deleted,
    /// A reading of a source that needs a professional opinion.
    Unresolved,
}

impl Status {
    /// Parse the status word used in `claims.tsv`.
    #[must_use]
    pub fn parse(word: &str) -> Option<Status> {
        Some(match word {
            "verified" => Status::Verified,
            "partly verified" => Status::PartlyVerified,
            "derived" => Status::Derived,
            "secondary" => Status::Secondary,
            "unverified" => Status::Unverified,
            "contradicted" => Status::Contradicted,
            "deleted" => Status::Deleted,
            "unresolved" => Status::Unresolved,
            _ => return None,
        })
    }

    /// The word printed in `CLAIMS.md`.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Status::Verified => "verified",
            Status::PartlyVerified => "partly verified",
            Status::Derived => "derived",
            Status::Secondary => "secondary",
            Status::Unverified => "unverified",
            Status::Contradicted => "contradicted",
            Status::Deleted => "deleted",
            Status::Unresolved => "unresolved",
        }
    }
}

/// One rendered claim: every field has had its `{{key}}` placeholders filled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    /// Row number shown in the table (1-based position in the file).
    pub id: String,
    /// The claim text.
    pub claim: String,
    /// Source (URL and locator) or calculation.
    pub source: String,
    /// Status, parsed from the first `;`-separated part of the status field.
    pub status: Status,
    /// Optional qualifier printed in parentheses after the status.
    pub note: String,
    /// File names under `model/sources/` (empty if no excerpt is committed).
    pub evidence: Vec<String>,
    /// Strings that must appear verbatim (modulo whitespace) in the excerpt.
    pub needles: Vec<String>,
}

/// Parse and render `claims.tsv`:
/// `claim<TAB>source<TAB>status<TAB>evidence<TAB>needles`. Rows are numbered
/// from 1 in file order.
/// `status` is `word` or `word; note` (and may itself be a placeholder, so a
/// status can be computed from model values). `evidence` is a file name or
/// `-`. `evidence` may list several files joined by ` + `. `needles` are separated
/// by ` || `. Lines starting with `#` and blank
/// lines are skipped.
///
/// # Errors
/// [`DocError::BadClaim`] for a malformed row or unknown status word, and
/// render errors for unknown placeholders.
pub fn parse_claims(tsv: &str, values: &[Value]) -> Result<Vec<Claim>, DocError> {
    let mut out = Vec::new();
    for line in tsv.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() != 5 {
            return Err(DocError::BadClaim(line.to_string()));
        }
        let r = |x: &str| render(x, values, &[]);
        let status_text = r(f[2])?;
        let (word, note) = match status_text.split_once(';') {
            Some((w, n)) => (w.trim(), n.trim().to_string()),
            None => (status_text.trim(), String::new()),
        };
        let status = Status::parse(word).ok_or_else(|| DocError::BadClaim(line.to_string()))?;
        let evidence: Vec<String> = match f[3].trim() {
            "-" | "" => Vec::new(),
            e => e.split(" + ").map(|x| x.trim().to_string()).collect(),
        };
        let needles = if f[4].trim().is_empty() || f[4].trim() == "-" {
            Vec::new()
        } else {
            f[4].split(" || ")
                .map(|n| r(n.trim()))
                .collect::<Result<Vec<_>, _>>()?
        };
        out.push(Claim {
            id: (out.len() + 1).to_string(),
            claim: r(f[0])?,
            source: r(f[1])?,
            status,
            note,
            evidence,
            needles,
        });
    }
    Ok(out)
}

/// Render claims as the markdown table used in `CLAIMS.md`.
#[must_use]
pub fn claims_table(claims: &[Claim]) -> String {
    let mut s = String::from("| # | Claim | Source (URL + locator) or calculation | Status |\n| --- | --- | --- | --- |\n");
    for c in claims {
        for part in [
            "| ",
            &c.id,
            " | ",
            &c.claim,
            " | ",
            &c.source,
            " | ",
            c.status.word(),
        ] {
            s.push_str(part);
        }
        if !c.note.is_empty() {
            s.push_str(" (");
            s.push_str(&c.note);
            s.push(')');
        }
        s.push_str(" |\n");
    }
    s
}

/// Split a committed source excerpt into its header and verbatim body.
/// The header must carry `# Source:`, `# URL:`, `# Accessed:` and `# Method:`
/// lines and end with a `---` line.
///
/// # Errors
/// A description of the missing part.
pub fn excerpt_body(text: &str) -> Result<&str, String> {
    let Some(cut) = text.find("\n---\n") else {
        return Err("no `---` line after the header".into());
    };
    let header = &text[..cut];
    for field in ["# Source:", "# URL:", "# Accessed:", "# Method:"] {
        if !header.lines().any(|l| l.starts_with(field)) {
            return Err(format!("header lacks `{field}`"));
        }
    }
    Ok(&text[cut + 5..])
}

fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `true` if `needle` occurs in `body`, comparing runs of whitespace as one
/// space (page extraction does not preserve layout whitespace).
#[must_use]
pub fn excerpt_contains(body: &str, needle: &str) -> bool {
    !needle.trim().is_empty() && squash(body).contains(&squash(needle))
}
