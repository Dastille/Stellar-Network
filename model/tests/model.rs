//! Behavioural tests.
use stellar_model::*;

#[test]
fn starlink_annual() {
    assert_eq!(service_cost_per_year(75.0, 1.0).unwrap(), 900.0);
    assert_eq!(
        service_cost_per_year(75.0, 1_000_000.0).unwrap(),
        900_000_000.0
    );
}

#[test]
fn pv_400w() {
    let p = pv(400.0).unwrap();
    assert!((p.annual_kwh - 465.2).abs() < 1e-9);
    assert!((p.value_cad_per_month - 4.341_866_666_666_667).abs() < 1e-9);
    assert!(
        p.dec_daily_kwh < p.daily_kwh,
        "December must be below the annual mean"
    );
}

#[test]
fn teg_reproduces_claim_and_heat_need() {
    let t = teg_chain(10.0, 60.0, 6.0).unwrap();
    assert!((t.kwh_e_per_day - 3.6).abs() < 1e-12);
    assert!((t.efficiency - 14.6 / 365.0).abs() < 1e-15);
    assert!((t.kwh_th_per_day - 90.0).abs() < 1e-9);
    // Heat is 25x the electricity at 4% efficiency.
    assert!((t.kwh_th_per_day / t.kwh_e_per_day - 25.0).abs() < 1e-9);
    assert!(t.area_dec_m2 > t.area_annual_m2);
}

#[test]
fn hydro_50lb() {
    let h = hydro(50.0).unwrap();
    assert!((h.kg_per_year - 50.0 * 0.453_592_37 * 365.0 / 7.0).abs() < 1e-9);
    assert!((h.total_kwh_per_day / h.light_kwh_per_day - 6.0).abs() < 1e-12);
}

#[test]
fn rejects_bad_inputs() {
    assert!(pv(0.0).is_err());
    assert!(teg_chain(5.0, -60.0, 6.0).is_err());
    assert!(hydro(f64::NAN).is_err());
    let e = service_cost_per_year(75.0, 0.0).unwrap_err();
    assert_eq!(
        e,
        ModelError::InvalidInput {
            name: "households",
            value: 0.0
        }
    );
    assert!(e.to_string().contains("households"));
}
