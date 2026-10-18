//! # stellar-model
//!
//! First-principles calculations behind `whitepaper/whitepaper.md` (v3.0).
//! Each sourced constant cites its source; [`render_paper_block`] produces
//! the table the paper embeds, and the tests assert the paper matches it.
//!
//! ```
//! let t = stellar_model::teg_chain(5.0, 60.0, 6.0).unwrap();
//! assert!((t.kwh_e_per_day - 1.8).abs() < 1e-12);
//! ```
#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::fmt;

// --------------------------------------------------------------------------
// Sourced inputs
// --------------------------------------------------------------------------

/// starlink.com/ca/service-plans, "Residential - 100 Mbps" CA$75/mo
/// (headless Chromium render, 2026-10-18 02:10 EDT).
pub const STARLINK_CA_RES100: f64 = 75.0;
/// Same page, "Residential - 200 Mbps" CA$115/mo.
pub const STARLINK_CA_RES200: f64 = 115.0;
/// Same page, "Residential - Max" CA$150/mo.
pub const STARLINK_CA_MAX: f64 = 150.0;

/// NRCan municip_potentiel-potential.csv, Ontario/Toronto, Annual, latitude tilt (kWh/kWp/yr).
pub const TORONTO_PV_KWH_PER_KWP_YR: f64 = 1163.0;
/// Same file, Ontario/Toronto, December, latitude tilt (kWh/kWp for the month).
pub const TORONTO_PV_DEC_KWH_PER_KWP: f64 = 51.0;
/// NRCan municip_kWh.csv, Ontario/Toronto, Annual, latitude tilt (kWh/m2/day).
pub const TORONTO_INSOL_ANNUAL: f64 = 4.25;
/// Same file, December, latitude tilt (kWh/m2/day).
pub const TORONTO_INSOL_DEC: f64 = 2.21;

/// OEB RPP Tier 1, effective 2025-11-01: 12.0 cents/kWh (commodity only).
pub const ONTARIO_TIER1_CAD_PER_KWH: f64 = 0.120;

/// Tecteg TEG1-12611-6.0 (56x56 mm, Bi2Te3) at hot 300 C / cold 30 C:
/// matched-load output 14.6 W. <https://tecteg.com/product/teg1-12611-6-0/>
pub const TEG_MODULE_W: f64 = 14.6;
/// Same datasheet: heat flow across the module ~365 W.
pub const TEG_MODULE_HEAT_W: f64 = 365.0;

/// Barbosa et al. 2015 (IJERPH 12(6):6879, doi:10.3390/ijerph120606879):
/// hydroponic lettuce yield 41 kg/m2/yr (Yuma, Arizona greenhouse model).
pub const HYDRO_YIELD_KG_M2_YR: f64 = 41.0;
/// Same paper: total energy 90,000 kJ per kg per year.
pub const HYDRO_ENERGY_KJ_PER_KG: f64 = 90_000.0;
/// Same paper: supplemental lighting share 15,000 kJ per kg per year.
pub const HYDRO_LIGHT_KJ_PER_KG: f64 = 15_000.0;

/// Exact: 1 lb = 0.453 592 37 kg.
pub const KG_PER_LB: f64 = 0.453_592_37;
/// Exact: 1 kWh = 3600 kJ.
pub const KJ_PER_KWH: f64 = 3600.0;

// --------------------------------------------------------------------------
// Errors
// --------------------------------------------------------------------------

/// Input validation error.
#[derive(Debug, Clone, PartialEq)]
pub enum ModelError {
    /// A parameter was not finite and strictly positive.
    InvalidInput {
        /// Parameter name.
        name: &'static str,
        /// Rejected value.
        value: f64,
    },
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::InvalidInput { name, value } => {
                write!(
                    f,
                    "invalid input `{name}` = {value}: must be finite and > 0"
                )
            }
        }
    }
}

impl std::error::Error for ModelError {}

fn positive(name: &'static str, value: f64) -> Result<f64, ModelError> {
    if value.is_finite() && value > 0.0 {
        Ok(value)
    } else {
        Err(ModelError::InvalidInput { name, value })
    }
}

// --------------------------------------------------------------------------
// Connectivity cost
// --------------------------------------------------------------------------

/// Annual service cost of `households` on a plan priced `monthly_cad`.
///
/// # Errors
/// [`ModelError::InvalidInput`] on non-positive input.
pub fn service_cost_per_year(monthly_cad: f64, households: f64) -> Result<f64, ModelError> {
    Ok(positive("monthly_cad", monthly_cad)? * 12.0 * positive("households", households)?)
}

// --------------------------------------------------------------------------
// PV
// --------------------------------------------------------------------------

/// PV figures for an array of `watts` W in Toronto.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pv {
    /// Annual kWh.
    pub annual_kwh: f64,
    /// Mean daily kWh.
    pub daily_kwh: f64,
    /// Mean daily kWh in December.
    pub dec_daily_kwh: f64,
    /// Monthly value at Tier 1, CAD.
    pub value_cad_per_month: f64,
}

/// PV chain: W/1000 x NRCan kWh/kWp.
///
/// # Errors
/// [`ModelError::InvalidInput`] on non-positive input.
pub fn pv(watts: f64) -> Result<Pv, ModelError> {
    let kwp = positive("watts", watts)? / 1000.0;
    let annual = kwp * TORONTO_PV_KWH_PER_KWP_YR;
    Ok(Pv {
        annual_kwh: annual,
        daily_kwh: annual / 365.0,
        dec_daily_kwh: kwp * TORONTO_PV_DEC_KWH_PER_KWP / 31.0,
        value_cad_per_month: annual * ONTARIO_TIER1_CAD_PER_KWH / 12.0,
    })
}

/// kWp needed to deliver `kwh_per_day` on the annual mean.
///
/// # Errors
/// [`ModelError::InvalidInput`] on non-positive input.
pub fn pv_kwp_for(kwh_per_day: f64) -> Result<f64, ModelError> {
    Ok(positive("kwh_per_day", kwh_per_day)? * 365.0 / TORONTO_PV_KWH_PER_KWP_YR)
}

// --------------------------------------------------------------------------
// TEG
// --------------------------------------------------------------------------

/// TEG chain for `n` generators of `w_each` W running `hours` at rated delta-T.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Teg {
    /// Module conversion efficiency (electric / heat).
    pub efficiency: f64,
    /// Datasheet modules per generator of `w_each` W (fractional).
    pub modules_per_generator: f64,
    /// Electrical kWh per day.
    pub kwh_e_per_day: f64,
    /// Heat that must cross the modules at 300 C hot side, kWh(th)/day.
    pub kwh_th_per_day: f64,
    /// Collector area if 100% of annual-mean latitude-tilt insolation became 300 C heat, m2.
    pub area_annual_m2: f64,
    /// Same for December, m2.
    pub area_dec_m2: f64,
    /// PV kWp that yields the same kWh/day on the annual mean.
    pub pv_equivalent_kwp: f64,
}

/// Evaluate the TEG claim from the module datasheet.
///
/// # Errors
/// [`ModelError::InvalidInput`] on non-positive input.
pub fn teg_chain(n: f64, w_each: f64, hours: f64) -> Result<Teg, ModelError> {
    let n = positive("n", n)?;
    let w = positive("w_each", w_each)?;
    let h = positive("hours", hours)?;
    let eff = TEG_MODULE_W / TEG_MODULE_HEAT_W;
    let kwh_e = n * w * h / 1000.0;
    let kwh_th = kwh_e / eff;
    Ok(Teg {
        efficiency: eff,
        modules_per_generator: w / TEG_MODULE_W,
        kwh_e_per_day: kwh_e,
        kwh_th_per_day: kwh_th,
        area_annual_m2: kwh_th / TORONTO_INSOL_ANNUAL,
        area_dec_m2: kwh_th / TORONTO_INSOL_DEC,
        pv_equivalent_kwp: pv_kwp_for(kwh_e)?,
    })
}

// --------------------------------------------------------------------------
// Hydroponics
// --------------------------------------------------------------------------

/// Hydroponic food chain for `lb_per_week` of greens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hydro {
    /// kg per year.
    pub kg_per_year: f64,
    /// Growing area, m2.
    pub area_m2: f64,
    /// Total energy, kWh/day (Barbosa total).
    pub total_kwh_per_day: f64,
    /// Lighting-only energy, kWh/day.
    pub light_kwh_per_day: f64,
}

/// Evaluate the hydroponics claim.
///
/// # Errors
/// [`ModelError::InvalidInput`] on non-positive input.
pub fn hydro(lb_per_week: f64) -> Result<Hydro, ModelError> {
    let kg = positive("lb_per_week", lb_per_week)? * KG_PER_LB * 365.0 / 7.0;
    Ok(Hydro {
        kg_per_year: kg,
        area_m2: kg / HYDRO_YIELD_KG_M2_YR,
        total_kwh_per_day: kg * HYDRO_ENERGY_KJ_PER_KG / KJ_PER_KWH / 365.0,
        light_kwh_per_day: kg * HYDRO_LIGHT_KJ_PER_KG / KJ_PER_KWH / 365.0,
    })
}

// --------------------------------------------------------------------------
// Rendering
// --------------------------------------------------------------------------

/// Start marker of the generated block.
pub const BEGIN: &str = "<!-- BEGIN GENERATED BY stellar-model (cargo run -q -p stellar-model) -->";
/// End marker of the generated block.
pub const END: &str = "<!-- END GENERATED -->";

fn money(x: f64) -> String {
    // Thousands separators for whole-dollar amounts.
    #[allow(clippy::cast_possible_truncation)]
    let n = x.round() as i64;
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 {
        format!("-${out}")
    } else {
        format!("${out}")
    }
}

/// Render the generated block embedded in the whitepaper.
///
/// # Errors
/// Propagates [`ModelError`] (cannot occur for the built-in inputs).
pub fn render_paper_block() -> Result<String, ModelError> {
    let mut o = String::new();
    let mut w = |s: String| {
        o.push_str(&s);
        o.push('\n');
    };
    w(BEGIN.to_string());
    w("| ID | Quantity | Value | Chain |".into());
    w("| --- | --- | ---: | --- |".into());
    for (id, name, m) in [
        ("K1", "Residential 100", STARLINK_CA_RES100),
        ("K2", "Residential 200", STARLINK_CA_RES200),
        ("K3", "Residential Max", STARLINK_CA_MAX),
    ] {
        w(format!(
            "| {id} | Starlink {name}, per household-year | {} | CA${m:.0} x 12 |",
            money(service_cost_per_year(m, 1.0)?)
        ));
    }
    for (id, n) in [("K4", 1_000.0), ("K5", 100_000.0), ("K6", 1_000_000.0)] {
        w(format!(
            "| {id} | Free service for {} households (Res 100), per year | {} | 900 x {} |",
            money(n).trim_start_matches('$'),
            money(service_cost_per_year(STARLINK_CA_RES100, n)?),
            money(n).trim_start_matches('$')
        ));
    }
    let p = pv(400.0)?;
    w(format!(
        "| P1 | 400 W panel, annual yield (Toronto) | {:.1} kWh/yr | 0.4 x 1163 |",
        p.annual_kwh
    ));
    w(format!(
        "| P2 | 400 W panel, mean daily | {:.2} kWh/day | P1 / 365 |",
        p.daily_kwh
    ));
    w(format!(
        "| P3 | 400 W panel, December daily | {:.2} kWh/day | 0.4 x 51 / 31 |",
        p.dec_daily_kwh
    ));
    w(format!(
        "| P4 | 400 W panel, value at Tier 1 | ${:.2}/mo | P1 x 0.120 / 12 |",
        p.value_cad_per_month
    ));
    w(format!(
        "| P5 | PV needed to save $50/mo at Tier 1 | {:.2} kWp | (600 / 0.120) / 1163 |",
        600.0 / ONTARIO_TIER1_CAD_PER_KWH / TORONTO_PV_KWH_PER_KWP_YR
    ));
    let eff = TEG_MODULE_W / TEG_MODULE_HEAT_W;
    w(format!(
        "| T1 | TEG module efficiency (datasheet) | {:.1}% | 14.6 W / 365 W |",
        100.0 * eff
    ));
    w(format!(
        "| T2 | Datasheet modules per \"60 W TEG\" | {:.2} | 60 / 14.6 |",
        60.0 / TEG_MODULE_W
    ));
    for (id, n) in [("T3", 5.0), ("T4", 10.0)] {
        let t = teg_chain(n, 60.0, 6.0)?;
        w(format!("| {id} | {n:.0} x 60 W TEG x 6 h at rated dT | {:.1} kWh(e)/day | {n:.0} x 60 x 6 / 1000 |", t.kwh_e_per_day));
        w(format!(
            "| {id}a | Heat through modules at 300 C | {:.0} kWh(th)/day | {id} / {eff:.3} |",
            t.kwh_th_per_day
        ));
        w(format!(
            "| {id}b | Collector area at 100% capture, annual mean | {:.1} m2 | {id}a / 4.25 |",
            t.area_annual_m2
        ));
        w(format!(
            "| {id}c | Collector area at 100% capture, December | {:.1} m2 | {id}a / 2.21 |",
            t.area_dec_m2
        ));
        w(format!(
            "| {id}d | PV giving the same kWh/day | {:.2} kWp | {id} x 365 / 1163 |",
            t.pv_equivalent_kwp
        ));
    }
    let h = hydro(50.0)?;
    w(format!(
        "| H1 | 50 lb/week greens | {:.0} kg/yr | 50 x 0.45359237 x 365/7 |",
        h.kg_per_year
    ));
    w(format!(
        "| H2 | Growing area at 41 kg/m2/yr | {:.1} m2 | H1 / 41 |",
        h.area_m2
    ));
    w(format!(
        "| H3 | Energy, Barbosa total (90,000 kJ/kg) | {:.1} kWh/day | H1 x 90000 / 3600 / 365 |",
        h.total_kwh_per_day
    ));
    w(format!(
        "| H4 | Energy, lighting only (15,000 kJ/kg) | {:.1} kWh/day | H1 x 15000 / 3600 / 365 |",
        h.light_kwh_per_day
    ));
    w(format!(
        "| H5 | 400 W panels to cover lighting only (annual mean) | {:.1} | H4 / P2 |",
        h.light_kwh_per_day / p.daily_kwh
    ));
    let load = 13.88 * 365.0;
    w(format!(
        "| L1 | 13.88 kWh/day load, annual | {load:.0} kWh/yr | 13.88 x 365 |"
    ));
    w(format!(
        "| L2 | Tariff implied by \"$2,000/yr\" | ${:.3}/kWh | 2000 / L1 |",
        2000.0 / load
    ));
    w(format!(
        "| L3 | Same load at Ontario Tier 1 | ${:.0}/yr | L1 x 0.120 |",
        load * ONTARIO_TIER1_CAD_PER_KWH
    ));
    o.push_str(END);
    Ok(o)
}

/// Key figures the README must quote verbatim (asserted by tests).
///
/// # Errors
/// Propagates [`ModelError`].
pub fn readme_facts() -> Result<Vec<String>, ModelError> {
    let p = pv(400.0)?;
    let t5 = teg_chain(5.0, 60.0, 6.0)?;
    let h = hydro(50.0)?;
    Ok(vec![
        format!("CA${STARLINK_CA_RES100:.0}"),
        money(service_cost_per_year(STARLINK_CA_RES100, 1.0)?),
        money(service_cost_per_year(STARLINK_CA_RES100, 1_000_000.0)?),
        format!("{:.2} kWh/day", p.daily_kwh),
        format!("${:.2}/mo", p.value_cad_per_month),
        format!("{:.0} kWh(th)/day", t5.kwh_th_per_day),
        format!("{:.2} kWp", t5.pv_equivalent_kwp),
        format!("{:.1} m2", h.area_m2),
        format!("{:.1} kWh/day", h.light_kwh_per_day),
    ])
}

/// Extract the generated block (markers included) from a markdown document.
#[must_use]
pub fn extract_block(doc: &str) -> Option<&str> {
    let start = doc.find(BEGIN)?;
    let end = doc[start..].find(END)? + start + END.len();
    Some(&doc[start..end])
}
