//! # stellar-model
//!
//! First-principles calculations behind `whitepaper/whitepaper.md` (v3.0).
//!
//! Each sourced constant cites its source. The whitepaper, `README.md` and
//! `CLAIMS.md` are rendered from `model/templates/` by [`render_paper`],
//! [`render_readme`] and [`render_claims`]: every model-derived number in
//! them, prose included, is a `{{key}}` placeholder filled from [`values`].
//! Every other number must be on `model/numbers-allowlist.tsv` with its
//! location and source. The tests assert that the committed files equal a
//! fresh render and that the allowlist covers every remaining number.
//!
//! ```
//! let t = stellar_model::teg_chain(5.0, 60.0, 6.0).unwrap();
//! assert!((t.kwh_e_per_day - 1.8).abs() < 1e-12);
//! ```
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]

pub mod docgen;

use docgen::{grouped, Claim, Cmp, DocError, Value};
use std::fmt;

// --------------------------------------------------------------------------
// Sourced inputs
// --------------------------------------------------------------------------

/// starlink.com/ca/service-plans, "Residential - 100 Mbps" CA$75/mo
/// (headless Chromium render, 2026-10-18 02:10 EDT; re-read 02:44 EDT).
pub const STARLINK_CA_RES100: f64 = 75.0;
/// Same page, "Residential - 200 Mbps" CA$115/mo.
pub const STARLINK_CA_RES200: f64 = 115.0;
/// Same page, "Residential - Max" CA$150/mo.
pub const STARLINK_CA_MAX: f64 = 150.0;

/// `NRCan` `municip_potentiel-potential.csv`, Ontario/Toronto, Annual, latitude tilt (kWh/kWp/yr).
pub const TORONTO_PV_KWH_PER_KWP_YR: f64 = 1163.0;
/// Same file, Ontario/Toronto, December, latitude tilt (kWh/kWp for the month).
pub const TORONTO_PV_DEC_KWH_PER_KWP: f64 = 51.0;
/// `NRCan` `municip_kWh.csv`, Ontario/Toronto, Annual, latitude tilt (kWh/m2/day).
pub const TORONTO_INSOL_ANNUAL: f64 = 4.25;
/// Same file, December, latitude tilt (kWh/m2/day).
pub const TORONTO_INSOL_DEC: f64 = 2.21;

/// OEB RPP Tier 1, effective 2025-11-01: 12.0 cents/kWh (commodity only).
pub const ONTARIO_TIER1_CAD_PER_KWH: f64 = 0.120;

/// Tecteg TEG1-12611-6.0 (56x56 mm) at hot 300 C / cold 30 C:
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

/// Date and time the Starlink prices were read (`America/New_York`).
pub const STARLINK_ACCESSED: &str = "2026-10-18 02:10 EDT";
/// Datasheet hot-side temperature, C (Tecteg TEG1-12611-6.0).
pub const TEG_HOT_C: f64 = 300.0;
/// Datasheet cold-side temperature, C (Tecteg TEG1-12611-6.0).
pub const TEG_COLD_C: f64 = 30.0;

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

/// PV chain: W/1000 x `NRCan` kWh/kWp.
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
// Ashlynn's figures (inputs to the checks; not sourced facts)
// --------------------------------------------------------------------------

/// Household counts for the free-service cost: her 1,000-user beta, 100K+
/// growth and 1M+ scale.
pub const HOUSEHOLDS: [f64; 3] = [1_000.0, 100_000.0, 1_000_000.0];
/// Starter solar panel, W (her v2.0 "400 W solar").
pub const PANEL_W: f64 = 400.0;
/// Monthly saving she targets, CAD (low end of her "$50-100/month").
pub const SAVING_TARGET_CAD_PER_MONTH: f64 = 50.0;
/// TEG counts (her "5-10 x 60 W TEGs").
pub const TEG_COUNTS: [f64; 2] = [5.0, 10.0];
/// Rated output of one of her TEGs, W.
pub const TEG_W_EACH: f64 = 60.0;
/// Hours per day at rated output implied by her 1.8-3.6 kWh/day.
pub const TEG_HOURS: f64 = 6.0;
/// Hydroponic greens, lb/week (her "50+ lb/week").
pub const HYDRO_LB_PER_WEEK: f64 = 50.0;
/// Household load, kWh/day (her v1/v2 figure; unsourced).
pub const HER_LOAD_KWH_PER_DAY: f64 = 13.88;
/// Modules in the stove co-generation example (a scenario choice, section 3).
pub const STOVE_TEG_MODULES: f64 = 4.0;
/// Household energy spend, CAD/yr (her "$2,000/yr"; unsourced).
pub const HER_SPEND_CAD_PER_YR: f64 = 2_000.0;

// --------------------------------------------------------------------------
// Rendering
// --------------------------------------------------------------------------

/// Start marker of the generated block.
pub const BEGIN: &str = "<!-- BEGIN GENERATED BY stellar-model (cargo run -q -p stellar-model) -->";
/// End marker of the generated block.
pub const END: &str = "<!-- END GENERATED -->";

/// Template of `whitepaper/whitepaper.md`.
pub const PAPER_TEMPLATE: &str = include_str!("../templates/whitepaper.md");
/// Template of `README.md`.
pub const README_TEMPLATE: &str = include_str!("../templates/README.md");
/// Template of `CLAIMS.md`.
pub const CLAIMS_TEMPLATE: &str = include_str!("../templates/CLAIMS.md");
/// Claims rows rendered into the `CLAIMS.md` table (see [`claims`]).
pub const CLAIMS_DATA: &str = include_str!("../claims.tsv");
/// Allowlist of numbers in the templates that are not model output.
pub const ALLOWLIST: &str = include_str!("../numbers-allowlist.tsv");

/// Error from rendering: either the model or the template failed.
#[derive(Debug, Clone, PartialEq)]
pub enum RenderError {
    /// A model calculation rejected its input.
    Model(ModelError),
    /// A template is malformed or names an unknown key.
    Doc(DocError),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::Model(e) => write!(f, "model: {e}"),
            RenderError::Doc(e) => write!(f, "template: {e}"),
        }
    }
}

impl std::error::Error for RenderError {}

impl From<ModelError> for RenderError {
    fn from(e: ModelError) -> Self {
        RenderError::Model(e)
    }
}

impl From<DocError> for RenderError {
    fn from(e: DocError) -> Self {
        RenderError::Doc(e)
    }
}

fn money(x: f64) -> String {
    let g = grouped(x, 0);
    match g.strip_prefix('-') {
        Some(rest) => format!("-${rest}"),
        None => format!("${g}"),
    }
}

struct Rows(String);

impl Rows {
    fn row(&mut self, id: &str, quantity: &str, value: &str, chain: &str) {
        for part in [
            "| ", id, " | ", quantity, " | ", value, " | ", chain, " |\n",
        ] {
            self.0.push_str(part);
        }
    }
}

fn starlink_rows(o: &mut Rows) -> Result<(), ModelError> {
    for (id, name, m) in [
        ("K1", "Residential 100", STARLINK_CA_RES100),
        ("K2", "Residential 200", STARLINK_CA_RES200),
        ("K3", "Residential Max", STARLINK_CA_MAX),
    ] {
        o.row(
            id,
            &format!("Starlink {name}, service only, per household-year"),
            &money(service_cost_per_year(m, 1.0)?),
            &format!("CA${m:.0} x 12"),
        );
    }
    let per_year = service_cost_per_year(STARLINK_CA_RES100, 1.0)?;
    for (id, n) in ["K4", "K5", "K6"].into_iter().zip(HOUSEHOLDS) {
        let count = grouped(n, 0);
        o.row(
            id,
            &format!("Free service for {count} households (Res 100), service only, per year"),
            &money(service_cost_per_year(STARLINK_CA_RES100, n)?),
            &format!("{} x {count}", grouped(per_year, 0)),
        );
    }
    Ok(())
}

fn pv_rows(o: &mut Rows) -> Result<(), ModelError> {
    let p = pv(PANEL_W)?;
    let kw = PANEL_W / 1000.0;
    let w = format!("{PANEL_W:.0} W panel");
    let t1 = ONTARIO_TIER1_CAD_PER_KWH;
    let nr = TORONTO_PV_KWH_PER_KWP_YR;
    o.row(
        "P1",
        &format!("{w}, annual yield (Toronto)"),
        &format!("{:.1} kWh/yr", p.annual_kwh),
        &format!("{kw} x {nr:.0}"),
    );
    o.row(
        "P2",
        &format!("{w}, mean daily"),
        &format!("{:.2} kWh/day", p.daily_kwh),
        "P1 / 365",
    );
    o.row(
        "P3",
        &format!("{w}, December daily"),
        &format!("{:.2} kWh/day", p.dec_daily_kwh),
        &format!("{kw} x {TORONTO_PV_DEC_KWH_PER_KWP:.0} / 31"),
    );
    o.row(
        "P4",
        &format!("{w}, value at Tier 1"),
        &format!("${:.2}/mo", p.value_cad_per_month),
        &format!("P1 x {t1:.3} / 12"),
    );
    let yearly = SAVING_TARGET_CAD_PER_MONTH * 12.0;
    o.row(
        "P5",
        &format!("PV needed to save ${SAVING_TARGET_CAD_PER_MONTH:.0}/mo at Tier 1"),
        &format!("{:.2} kWp", yearly / t1 / nr),
        &format!("({yearly:.0} / {t1:.3}) / {nr:.0}"),
    );
    Ok(())
}

fn teg_rows(o: &mut Rows) -> Result<(), ModelError> {
    let eff = TEG_MODULE_W / TEG_MODULE_HEAT_W;
    let (hot, mw, hw, we) = (TEG_HOT_C, TEG_MODULE_W, TEG_MODULE_HEAT_W, TEG_W_EACH);
    o.row(
        "T1",
        "TEG module efficiency (datasheet)",
        &format!("{:.1}%", 100.0 * eff),
        &format!("{mw} W / {hw:.0} W"),
    );
    o.row(
        "T2",
        &format!("Datasheet modules per \"{we:.0} W TEG\""),
        &format!("{:.2}", we / mw),
        &format!("{we:.0} / {mw}"),
    );
    for (id, n) in ["T3", "T4"].into_iter().zip(TEG_COUNTS) {
        let t = teg_chain(n, we, TEG_HOURS)?;
        let h = TEG_HOURS;
        o.row(
            id,
            &format!("{n:.0} x {we:.0} W TEG x {h:.0} h at rated dT"),
            &format!("{:.1} kWh(e)/day", t.kwh_e_per_day),
            &format!("{n:.0} x {we:.0} x {h:.0} / 1000"),
        );
        o.row(
            &format!("{id}a"),
            &format!("Heat through modules at {hot:.0} C"),
            &format!("{:.0} kWh(th)/day", t.kwh_th_per_day),
            &format!("{id} / {eff:.3}"),
        );
        o.row(
            &format!("{id}b"),
            "Collector area at 100% capture, annual mean",
            &format!("{:.1} m2", t.area_annual_m2),
            &format!("{id}a / {TORONTO_INSOL_ANNUAL}"),
        );
        o.row(
            &format!("{id}c"),
            "Collector area at 100% capture, December",
            &format!("{:.1} m2", t.area_dec_m2),
            &format!("{id}a / {TORONTO_INSOL_DEC}"),
        );
        o.row(
            &format!("{id}d"),
            "PV giving the same kWh/day",
            &format!("{:.2} kWp", t.pv_equivalent_kwp),
            &format!("{id} x 365 / {TORONTO_PV_KWH_PER_KWP_YR:.0}"),
        );
    }
    Ok(())
}

fn food_and_load_rows(o: &mut Rows) -> Result<(), ModelError> {
    let h = hydro(HYDRO_LB_PER_WEEK)?;
    let p = pv(PANEL_W)?;
    let lb = HYDRO_LB_PER_WEEK;
    o.row(
        "H1",
        &format!("{lb:.0} lb/week greens"),
        &format!("{:.0} kg/yr", h.kg_per_year),
        &format!("{lb:.0} x {KG_PER_LB} x 365/7"),
    );
    o.row(
        "H2",
        &format!("Growing area at {HYDRO_YIELD_KG_M2_YR:.0} kg/m2/yr"),
        &format!("{:.1} m2", h.area_m2),
        &format!("H1 / {HYDRO_YIELD_KG_M2_YR:.0}"),
    );
    o.row(
        "H3",
        &format!(
            "Energy, Barbosa total ({} kJ/kg)",
            grouped(HYDRO_ENERGY_KJ_PER_KG, 0)
        ),
        &format!("{:.1} kWh/day", h.total_kwh_per_day),
        &format!("H1 x {HYDRO_ENERGY_KJ_PER_KG:.0} / {KJ_PER_KWH:.0} / 365"),
    );
    o.row(
        "H4",
        &format!(
            "Energy, lighting only ({} kJ/kg)",
            grouped(HYDRO_LIGHT_KJ_PER_KG, 0)
        ),
        &format!("{:.1} kWh/day", h.light_kwh_per_day),
        &format!("H1 x {HYDRO_LIGHT_KJ_PER_KG:.0} / {KJ_PER_KWH:.0} / 365"),
    );
    o.row(
        "H5",
        &format!("{PANEL_W:.0} W panels to cover lighting only (annual mean)"),
        &format!("{:.1}", h.light_kwh_per_day / p.daily_kwh),
        "H4 / P2",
    );
    let d = HER_LOAD_KWH_PER_DAY;
    let load = d * 365.0;
    let spend = HER_SPEND_CAD_PER_YR;
    o.row(
        "L1",
        &format!("{d} kWh/day load, annual"),
        &format!("{load:.0} kWh/yr"),
        &format!("{d} x 365"),
    );
    o.row(
        "L2",
        &format!("Tariff implied by \"{}/yr\"", money(spend)),
        &format!("${:.3}/kWh", spend / load),
        &format!("{spend:.0} / L1"),
    );
    o.row(
        "L3",
        "Same load at Ontario Tier 1",
        &format!("${:.0}/yr", load * ONTARIO_TIER1_CAD_PER_KWH),
        &format!("L1 x {ONTARIO_TIER1_CAD_PER_KWH:.3}"),
    );
    Ok(())
}

/// Render the generated block embedded in the whitepaper (markers included).
///
/// # Errors
/// Propagates [`ModelError`] (cannot occur for the built-in inputs).
pub fn render_paper_block() -> Result<String, ModelError> {
    let mut o = Rows(String::new());
    o.0.push_str(BEGIN);
    o.0.push('\n');
    o.0.push_str("| ID | Quantity | Value | Chain |\n| --- | --- | ---: | --- |\n");
    starlink_rows(&mut o)?;
    pv_rows(&mut o)?;
    teg_rows(&mut o)?;
    food_and_load_rows(&mut o)?;
    o.0.push_str(END);
    Ok(o.0)
}

fn input(key: &'static str, text: String) -> Value {
    Value {
        key,
        text,
        derived: false,
    }
}

fn derived(key: &'static str, text: String) -> Value {
    Value {
        key,
        text,
        derived: true,
    }
}

/// Every value a template may reference, keyed by name.
///
/// # Errors
/// Propagates [`ModelError`] (cannot occur for the built-in inputs).
pub fn values() -> Result<Vec<Value>, ModelError> {
    let p = pv(PANEL_W)?;
    let t5 = teg_chain(TEG_COUNTS[0], TEG_W_EACH, TEG_HOURS)?;
    let t10 = teg_chain(TEG_COUNTS[1], TEG_W_EACH, TEG_HOURS)?;
    let h = hydro(HYDRO_LB_PER_WEEK)?;
    let load = HER_LOAD_KWH_PER_DAY * 365.0;
    let per_hh = service_cost_per_year(STARLINK_CA_RES100, 1.0)?;
    let mut v = vec![
        // Sourced inputs.
        input("starlink.res100", format!("{STARLINK_CA_RES100:.0}")),
        input("starlink.res200", format!("{STARLINK_CA_RES200:.0}")),
        input("starlink.max", format!("{STARLINK_CA_MAX:.0}")),
        input("starlink.accessed", STARLINK_ACCESSED.to_string()),
        input("nrcan.kwh_per_kwp", grouped(TORONTO_PV_KWH_PER_KWP_YR, 0)),
        input(
            "nrcan.kwh_per_kwp_raw",
            format!("{TORONTO_PV_KWH_PER_KWP_YR:.0}"),
        ),
        input("nrcan.dec", format!("{TORONTO_PV_DEC_KWH_PER_KWP:.0}")),
        input("nrcan.insol", format!("{TORONTO_INSOL_ANNUAL}")),
        input("nrcan.insol_dec", format!("{TORONTO_INSOL_DEC}")),
        input(
            "oeb.tier1_cents",
            format!("{:.1}", ONTARIO_TIER1_CAD_PER_KWH * 100.0),
        ),
        input("teg.module_w", format!("{TEG_MODULE_W}")),
        input("teg.heat_w", format!("{TEG_MODULE_HEAT_W:.0}")),
        input("teg.hot_c", format!("{TEG_HOT_C:.0}")),
        input("teg.cold_c", format!("{TEG_COLD_C:.0}")),
        input("teg.dt_k", format!("{:.0}", TEG_HOT_C - TEG_COLD_C)),
        input("hydro.yield", format!("{HYDRO_YIELD_KG_M2_YR:.0}")),
        input("hydro.energy", grouped(HYDRO_ENERGY_KJ_PER_KG, 0)),
        input("hydro.light", grouped(HYDRO_LIGHT_KJ_PER_KG, 0)),
        // Derived.
        derived("k1.per_hh", grouped(per_hh, 0)),
        derived("p1.annual", format!("{:.1}", p.annual_kwh)),
        derived("p2.daily", format!("{:.2}", p.daily_kwh)),
        derived("p3.dec", format!("{:.2}", p.dec_daily_kwh)),
        derived("p4.month", format!("{:.2}", p.value_cad_per_month)),
        derived(
            "p5.kwp",
            format!(
                "{:.2}",
                SAVING_TARGET_CAD_PER_MONTH * 12.0
                    / ONTARIO_TIER1_CAD_PER_KWH
                    / TORONTO_PV_KWH_PER_KWP_YR
            ),
        ),
        derived("t1.eff", format!("{:.1}", 100.0 * t5.efficiency)),
        derived("t3.kwh_e", format!("{:.1}", t5.kwh_e_per_day)),
        derived("t4.kwh_e", format!("{:.1}", t10.kwh_e_per_day)),
        derived("t3.kwh_th", format!("{:.0}", t5.kwh_th_per_day)),
        derived("t4.kwh_th", format!("{:.0}", t10.kwh_th_per_day)),
        derived("t3.area", format!("{:.1}", t5.area_annual_m2)),
        derived("t4.area_dec", format!("{:.1}", t10.area_dec_m2)),
        derived("t3.pv", format!("{:.2}", t5.pv_equivalent_kwp)),
        derived("t4.pv", format!("{:.2}", t10.pv_equivalent_kwp)),
        derived(
            "t.four_modules_w",
            format!("{:.0}", STOVE_TEG_MODULES * TEG_MODULE_W),
        ),
        derived(
            "t.four_modules_heat_kw",
            format!("{:.2}", STOVE_TEG_MODULES * TEG_MODULE_HEAT_W / 1000.0),
        ),
        derived("h2.area", format!("{:.1}", h.area_m2)),
        derived("h3.total", format!("{:.1}", h.total_kwh_per_day)),
        derived("h4.light", format!("{:.1}", h.light_kwh_per_day)),
        derived(
            "h5.panels",
            format!("{:.1}", h.light_kwh_per_day / p.daily_kwh),
        ),
        derived("l2.tariff", format!("{:.3}", HER_SPEND_CAD_PER_YR / load)),
        derived(
            "l2.ratio",
            format!(
                "{:.1}",
                HER_SPEND_CAD_PER_YR / load / ONTARIO_TIER1_CAD_PER_KWH
            ),
        ),
    ];
    for (key, n) in [
        ("k4.total", HOUSEHOLDS[0]),
        ("k5.total", HOUSEHOLDS[1]),
        ("k6.total", HOUSEHOLDS[2]),
    ] {
        v.push(derived(
            key,
            grouped(service_cost_per_year(STARLINK_CA_RES100, n)?, 0),
        ));
    }
    v.extend(her_inputs());
    v.extend(comparisons(
        p.value_cad_per_month,
        HER_SPEND_CAD_PER_YR / load,
    ));
    Ok(v)
}

/// Ashlynn's figures used as model inputs (unsourced unless stated).
fn her_inputs() -> Vec<Value> {
    vec![
        input("her.households_beta", grouped(HOUSEHOLDS[0], 0)),
        input("her.panel_w", format!("{PANEL_W:.0}")),
        input(
            "her.saving_low",
            format!("{SAVING_TARGET_CAD_PER_MONTH:.0}"),
        ),
        input("her.teg_low", format!("{:.0}", TEG_COUNTS[0])),
        input("her.teg_high", format!("{:.0}", TEG_COUNTS[1])),
        input("her.teg_w", format!("{TEG_W_EACH:.0}")),
        input("her.teg_hours", format!("{TEG_HOURS:.0}")),
        input("her.lb_week", format!("{HYDRO_LB_PER_WEEK:.0}")),
        input("her.load", format!("{HER_LOAD_KWH_PER_DAY}")),
        input("her.spend", grouped(HER_SPEND_CAD_PER_YR, 0)),
        input("stove.modules", format!("{STOVE_TEG_MODULES:.0}")),
    ]
}

/// Comparison words and verdicts generated from the values they compare, so
/// that changing an input cannot leave "exceeds" or "contradicted" stale.
fn comparisons(p4_month: f64, l2_tariff: f64) -> Vec<Value> {
    let saving = Cmp::of(p4_month, SAVING_TARGET_CAD_PER_MONTH, 2);
    let tariff = Cmp::of(l2_tariff, ONTARIO_TIER1_CAD_PER_KWH, 3);
    let short = saving == Cmp::Below;
    let pick = |c: bool, a: &str, b: &str| (if c { a } else { b }).to_string();
    vec![
        derived("cmp.p4_target.exceeds", saving.exceeds().to_string()),
        derived("cmp.l2_tier1.exceeds", tariff.exceeds().to_string()),
        derived("verdict.saving", pick(short, "No", "Yes")),
        derived(
            "status.saving_v2",
            pick(
                short,
                "contradicted; for the starter kit",
                "unverified; the starter kit meets the low end",
            ),
        ),
        derived(
            "status.tariff_v2",
            pick(
                tariff == Cmp::Above,
                "unverified; implied tariff exceeds the Ontario first-tier price",
                "unverified; no source",
            ),
        ),
    ]
}

/// The `CLAIMS.md` rows, parsed from [`CLAIMS_DATA`] with every placeholder
/// rendered from [`values`].
///
/// # Errors
/// [`RenderError`] if the model fails or a claims row is malformed.
pub fn claims() -> Result<Vec<Claim>, RenderError> {
    Ok(docgen::parse_claims(CLAIMS_DATA, &values()?)?)
}

fn render_doc(template: &str) -> Result<String, RenderError> {
    let block = render_paper_block()?;
    let table = docgen::claims_table(&claims()?);
    Ok(docgen::render(
        template,
        &values()?,
        &[("block", block.as_str()), ("claims_table", table.as_str())],
    )?)
}

/// Render `whitepaper/whitepaper.md` from its template.
///
/// # Errors
/// [`RenderError`] if the model or the template fails.
pub fn render_paper() -> Result<String, RenderError> {
    render_doc(PAPER_TEMPLATE)
}

/// Render `README.md` from its template.
///
/// # Errors
/// [`RenderError`] if the model or the template fails.
pub fn render_readme() -> Result<String, RenderError> {
    render_doc(README_TEMPLATE)
}

/// Render `CLAIMS.md` from its template.
///
/// # Errors
/// [`RenderError`] if the model or the template fails.
pub fn render_claims() -> Result<String, RenderError> {
    render_doc(CLAIMS_TEMPLATE)
}

/// Extract the generated block (markers included) from a markdown document.
#[must_use]
pub fn extract_block(doc: &str) -> Option<&str> {
    let start = doc.find(BEGIN)?;
    let end = doc[start..].find(END)? + start + END.len();
    Some(&doc[start..end])
}
