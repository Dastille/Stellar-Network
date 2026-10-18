# CLAIMS — Stellar Network whitepaper v3.0

The statuses are the same as in the CHOAM repository:
- **verified**: confirmed against the primary source.
- **derived**: computed by `model/` (run `cargo test`).
- **secondary**: confirmed through a secondary copy only.
- **unverified**: no source found.
- **contradicted**: the evidence disagrees.
- **deleted**: removed.

Access date for every source: 2026-10-18 (America/New_York).

| # | Claim | Source (URL + locator) or calculation | Status |
| --- | --- | --- | --- |
| 1 | Starlink Canada Residential 100 / 200 / Max = CA$75 / $115 / $150 per month | https://www.starlink.com/ca/service-plans "MONTHLY SERVICE PLANS" (headless Chromium, 02:10 EDT); matches Cloudy's 2026-10-18 log | verified |
| 2 | "No upfront hardware cost in select areas" | https://www.starlink.com/ca/residential hero text | verified |
| 3 | Starlink serves 160+ countries, territories and other markets | https://www.starlink.com/ca/residential | verified |
| 4 | Free service: $900/household-yr; $900,000 / $90,000,000 / $900,000,000 for 1k / 100k / 1M households | `model` K1–K6 | derived |
| 5 | $499 kit price (v2.2 README) | not shown on the pages read | unverified (not used) |
| 6 | Toronto PV 1163 kWh/kWp/yr; December 51 kWh/kWp | NRCan municip_potentiel-potential.csv, Ontario/Toronto, latitude tilt | verified |
| 7 | Toronto insolation 4.25 (annual) / 2.21 (Dec) kWh/m²/day | NRCan municip_kWh.csv, Ontario/Toronto, latitude tilt | verified |
| 8 | 400 W → 465.2 kWh/yr, 1.27 kWh/day, 0.66 kWh/day in Dec, $4.65/mo | `model` P1–P4 | derived |
| 9 | Ontario Tier 1 12.0 ¢/kWh | OEB electricity rates, Tiered table https://oeb.ca/consumer-information-and-protection/electricity-rates | verified |
| 10 | TEG module TEG1-12611-6.0: 14.6 W matched load, ~365 W heat flow at 300/30 °C; US$56 (1–9 pcs) | https://tecteg.com/product/teg1-12611-6-0/ product description table | verified |
| 11 | v2.0 "5–10 × 60 W TEGs … 1.8–3.6 kWh/day" | = nameplate × 6 h (T3, T4); requires 45–90 kWh(th)/day at 300 °C (T3a, T4a) and ≥10.6–40.7 m² of ideal collectors (T3b–T4c) | contradicted as a household primary supply; arithmetic reproduced |
| 12 | Same energy from 0.56–1.13 kWp PV | `model` T3d, T4d | derived |
| 13 | Hydroponic lettuce: 41 kg/m²/yr; 90,000 kJ/kg/yr total; 15,000 lighting | Barbosa et al. 2015, Abstract and §3 https://doi.org/10.3390/ijerph120606879 | verified |
| 14 | v2.0 "50+ lbs/week greens" → 1183 kg/yr, 28.8 m², 81.0 kWh/day total, 13.5 lighting | `model` H1–H5 | derived (feasible only with that energy) |
| 15 | v2.0 "13.88 kWh/day usage costs $2,000/year" | no source; implies $0.395/kWh (L2) vs 12.0 ¢ Tier 1 | unverified; tariff inconsistent with Ontario |
| 16 | v1/v2 "households spending $200/month on basics" | no source | unverified |
| 17 | v2.0 "save $50–100/month", "$600/year" | 400 W saves $4.65/mo (P4); $50/mo needs 4.30 kWp (P5) | contradicted for the starter kit |
| 18 | v2.0 "1.5M households lack reliable power/internet" | no source; CRTC CTMR 2026: >96% have USO access; rural >80% | unverified (replaced) |
| 19 | CRTC USO = 50/10 Mbps + unlimited option | CRTC 2016-496; CTMR 2026 §7 "universal service objective" | verified |
| 20 | Utility tokens reviewed by CSA mostly involved securities; four-part investment-contract test | CSA Staff Notice 46-308, "When an offering of tokens may or may not involve an offering of securities" | verified |
| 21 | Starlink average power draw (v2.2 front-end 0.8 kWh/day) | not found on starlink.com pages read | unverified (not used) |
| 22 | Methane backup feasibility | none found | unverified |
| 23 | Starship Mars Habitat Modules exist as a product | none found | unverified (vision) |
| 24 | v2.0 "$1B+ from 1M users"; "Green impact 20–30%" | no source | unverified |
| 25 | Net metering pays credits, not cash | OEB net metering page | verified |

## Spot-checks (3)

1. **Item 1, Starlink price.** Re-rendered https://www.starlink.com/ca/service-plans at 2026-10-18T06:10:49Z (02:10 EDT). The text reads "Residential - 100 Mbps CA$75 /mo · Residential - 200 Mbps CA$115 /mo · Residential - Max CA$150 /mo". **Pass**, and it agrees with Cloudy's log.
2. **Item 10, TEG datasheet.** The tecteg.com table reads "Match Load Output Watts (W) 14.6" and "Heat Flow Across the module Watts (W) ~365". 14.6/365 = 0.040, so the model's T1 of 4.0% matches. **Pass.**
3. **Item 13, hydroponics.** The Barbosa 2015 abstract reads "41 ± 6.1 kg/m²/y … 90,000 ± 11,000 kJ/kg/y", and §3 reads "supplemental artificial lighting at 15,000 ± 2100 kJ/kg/y". **Pass.**
