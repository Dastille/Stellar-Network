# Stellar Network: A Self-Sufficiency Path from a Free Starlink Kit to Habitat and Food (v3.0)

*Author: Ashlynn ([Dastille](https://github.com/Dastille)) · 2026-10-18 · License: AGPL-3.0*
*Her v2.0 title (2025-08-17) was "Empowering Self-Sufficiency with xAI-Driven Innovation". v3.0 keeps her v2.0 vision as the thesis. Each number is sourced or derived in `model/` and asserted by `cargo test`. Appendix A lists what changed.*

## Abstract

**Problem.** Rural and self-sufficiency-seeking households, worldwide (her v2.0 scope), lack affordable connectivity, power and food security. In Canada, over 80% of the rural population now has broadband at the CRTC's 50/10 Mbps objective [CRTC-2026], which leaves the rest underserved. **Proposal (Ashlynn, v2.0).** Give each household a **free, fully installed Starlink kit**. The household then **earns tokens** by sharing energy, data and compute, and spends them on expansions: 400 W solar, a used 10 kWh EV battery, **thermoelectric generators (TEGs)** with heat storage, and **methane backup**. Over time the path extends to **Starship-class habitat modules** of 200–500 sq ft and **hydroponic food** of about 50 lb/week of greens, all installed by volunteers and governed by guardian software and user vetoes. **Method.** We run a five-step requirements review and a first-principles derivation of each element. The inputs are the live starlink.com/ca price list, NRCan solar tables, the OEB tariff, a commercial TEG datasheet [Tecteg], and a peer-reviewed hydroponics energy model [Barbosa-2015]. **Results.** Free service costs a funder CA$900 per household-year, or $900,000,000/yr at 1M households (K1, K6). A 400 W panel yields 1.27 kWh/day in Toronto (0.66 in December), worth $4.65/mo. Her TEG figure of 1.8–3.6 kWh/day is the nameplate arithmetic of 5–10 × 60 W × 6 h. At the datasheet's 4.0% efficiency it needs 45–90 kWh of 300 °C heat per day, while 0.56–1.13 kWp of PV delivers the same energy. 50 lb/week of hydroponic greens needs about 28.8 m² and 13.5 kWh/day for lighting alone. An earn-and-redeem token is likely a security under Canadian guidance [CSA-46-308]. All of her ideas are kept. Section 3 states which are feasible now and what would have to change for the rest.

## 1. Problem statement and definitions

Deliver to a household, at no upfront cost to it, (a) broadband meeting at least the CRTC 50/10 Mbps objective [CRTC-2016-496], (b) a staged path to household energy self-supply, and (c) a long-term path to shelter and food. Every stage must be lawful, safe, and financed by someone the paper names.

- **Kit.** A Starlink terminal and router, installed. Starlink's own page currently offers "No upfront hardware cost in select areas" [Starlink-CA].
- **Free (to household).** The household pays $0. The paper always names who pays instead: a funder, a partner or a public program.
- **Expansion.** An optional module after the kit: PV, battery, TEG, methane, habitat or hydroponics.
- **Token (her mechanism).** A unit earned per contribution (example: 1 token per kWh shared) and spent on expansions, with a burn rate of 5–15%, capped.
- **Guardian.** Software that optimizes usage, adjusts token supply and flags fraud. It is a design requirement and has not been built.
- **Feasible now / needs change.** These classifications are used in section 3. Claim statuses are defined in [CLAIMS.md](../CLAIMS.md).

## 2. Five-step requirements review

| Requirement (source) | Question | Decision |
| --- | --- | --- |
| Free Starlink kit (Ashlynn v1/v2) | Who pays service after hardware? | **Keep.** Service is $900/household-yr (K1). The funder must be named: partner, grant or public fund (section 5) |
| Token earned by kWh/data/compute (Ashlynn) | Is a redeemable, capped token a security? | **Keep the goal; redesign** (section 6). Likely a security as designed [CSA-46-308] |
| 5–10 × 60 W TEGs, 1.8–3.6 kWh/day (Ashlynn) | What heat input does it need? | **Keep as vision.** Infeasible as primary supply (T3–T4). Feasible as a stove co-generator (section 3) |
| Methane backup (Ashlynn) | Feedstock, climate, code? | **Keep; unverified.** No primary source yet on household digesters in Canadian winters |
| Starship Mars Habitat Modules (Ashlynn v2.0) | Does a product exist? | **Keep as long-term vision.** We found no SpaceX habitat product listing. The terrestrial analog is [CHOAM](https://github.com/Dastille/CHOAM-initiative) |
| Hydroponic food, 50+ lb/week (Ashlynn) | Area and energy? | **Keep.** 28.8 m², 13.5–81.0 kWh/day (H2–H4). Scale it to the household's power |
| Global scope (Ashlynn v2.0) | Is the service available outside Canada? | **Keep.** Starlink states service in 160+ countries and territories [Starlink-CA]. Prices here are Canadian only |
| "xAI self-regulation avoids red tape" (Ashlynn v2.0) | Does self-regulation displace law? | **Flagged.** No. PIPEDA [PIPEDA], securities law [CSA-46-308] and grid-connection rules [OEB-NM] still apply |
| Cash BOM replacing the vision (v2.2 rewrite, not hers) | Did Ashlynn supersede her v2.0? | **Delete the demotion.** v2.0 was her latest version, so it stands as the thesis |

**Delete:** unsourced figures (see Appendix A). **Simplify:** Starlink first, then PV, because PV gives the most kWh per dollar of the expansions evaluated (T3d vs T3a). **Accelerate:** the 1,000-household Starlink-only beta needs no new hardware. **Automate last:** guardian software and token accounting come after the beta has measured real flows. The calculations themselves are already automated (`cargo test`).

## 3. Vision vs feasible

| Her idea | Evidence (derived below) | Feasible now? | What would have to change |
| --- | --- | --- | --- |
| Free installed Starlink kit | Hardware $0 in select areas [Starlink-CA]; service CA$75/mo | **Yes, if funded** | A funder for $900/household-yr (K4–K6) |
| Earn-and-spend token | CSA: "utility" tokens reviewed were mostly securities [CSA-46-308] | **Not as designed** | Make credits non-transferable, not tradable and not redeemable for cash, with no profit expectation. Even then, get legal review |
| TEGs, 1.8–3.6 kWh/day | Needs 45–90 kWh(th)/day at 300 °C (T3a–T4a), or 10.6–40.7 m² of perfect collectors (T3b–T4c) | **No** as primary supply | Use waste heat already produced (wood stove): 4 modules ≈ 58 W at rated ΔT. Or wait for TEG efficiency well above 4% |
| Methane backup | No primary source yet | **Unverified** | Cold-climate digester data; gas-code compliance |
| Starship Mars Habitat Modules | No product found | **No** (no product) | A SpaceX product, or CHOAM units as the terrestrial step |
| Hydroponic food, 50 lb/week | 28.8 m²; lighting 13.5 kWh/day = 10.6 × 400 W panels (H5); total 81.0 kWh/day in the Arizona model | **Partly** (smaller scale) | Size food to the household's energy, e.g. a greenhouse with daylight |
| Save $50–100/month | A 400 W panel saves $4.65/mo; $50/mo needs 4.30 kWp (P5) | **No** for the starter kit | Larger PV, or savings counted against the household's prior internet bill (unsourced) |
| Global reach | 160+ markets [Starlink-CA] | **Yes** (service exists) | Per-country price and law |
| Anti-corruption: blockchain, user vetoes | — | Design goal | Public ledger of rule changes; vetoes as specified governance |
| SpaceX/xAI partnership (her pitch) | Independent proposal; not affiliated or endorsed | Proposal stands | A partner's agreement |

## 4. Derivations (generated; do not edit by hand)

Chains:
- **PV:** kWh = kWp × NRCan kWh/kWp (Toronto, latitude tilt: 1,163/yr; December 51/month) [NRCan-PV].
- **TEG:** heat = electricity / efficiency, with efficiency = 14.6 W / 365 W at hot side 300 °C and cold side 30 °C [Tecteg]. Collector area = heat / NRCan insolation (4.25 kWh/m²/day annual; 2.21 December). This assumes 100% capture, so it is a lower bound.
- **Hydroponics:** yield 41 kg/m²/yr and energy 90,000 kJ/kg (lighting 15,000) [Barbosa-2015].
- **Tariff:** OEB Tier 1 at 12.0 ¢/kWh [OEB-RPP].
- **Starlink:** CA$75 / $115 / $150 per month, from starlink.com/ca/service-plans, read 2026-10-18 02:10 EDT [Starlink-CA].

<!-- BEGIN GENERATED BY stellar-model (cargo run -q -p stellar-model) -->
| ID | Quantity | Value | Chain |
| --- | --- | ---: | --- |
| K1 | Starlink Residential 100, per household-year | $900 | CA$75 x 12 |
| K2 | Starlink Residential 200, per household-year | $1,380 | CA$115 x 12 |
| K3 | Starlink Residential Max, per household-year | $1,800 | CA$150 x 12 |
| K4 | Free service for 1,000 households (Res 100), per year | $900,000 | 900 x 1,000 |
| K5 | Free service for 100,000 households (Res 100), per year | $90,000,000 | 900 x 100,000 |
| K6 | Free service for 1,000,000 households (Res 100), per year | $900,000,000 | 900 x 1,000,000 |
| P1 | 400 W panel, annual yield (Toronto) | 465.2 kWh/yr | 0.4 x 1163 |
| P2 | 400 W panel, mean daily | 1.27 kWh/day | P1 / 365 |
| P3 | 400 W panel, December daily | 0.66 kWh/day | 0.4 x 51 / 31 |
| P4 | 400 W panel, value at Tier 1 | $4.65/mo | P1 x 0.120 / 12 |
| P5 | PV needed to save $50/mo at Tier 1 | 4.30 kWp | (600 / 0.120) / 1163 |
| T1 | TEG module efficiency (datasheet) | 4.0% | 14.6 W / 365 W |
| T2 | Datasheet modules per "60 W TEG" | 4.11 | 60 / 14.6 |
| T3 | 5 x 60 W TEG x 6 h at rated dT | 1.8 kWh(e)/day | 5 x 60 x 6 / 1000 |
| T3a | Heat through modules at 300 C | 45 kWh(th)/day | T3 / 0.040 |
| T3b | Collector area at 100% capture, annual mean | 10.6 m2 | T3a / 4.25 |
| T3c | Collector area at 100% capture, December | 20.4 m2 | T3a / 2.21 |
| T3d | PV giving the same kWh/day | 0.56 kWp | T3 x 365 / 1163 |
| T4 | 10 x 60 W TEG x 6 h at rated dT | 3.6 kWh(e)/day | 10 x 60 x 6 / 1000 |
| T4a | Heat through modules at 300 C | 90 kWh(th)/day | T4 / 0.040 |
| T4b | Collector area at 100% capture, annual mean | 21.2 m2 | T4a / 4.25 |
| T4c | Collector area at 100% capture, December | 40.7 m2 | T4a / 2.21 |
| T4d | PV giving the same kWh/day | 1.13 kWp | T4 x 365 / 1163 |
| H1 | 50 lb/week greens | 1183 kg/yr | 50 x 0.45359237 x 365/7 |
| H2 | Growing area at 41 kg/m2/yr | 28.8 m2 | H1 / 41 |
| H3 | Energy, Barbosa total (90,000 kJ/kg) | 81.0 kWh/day | H1 x 90000 / 3600 / 365 |
| H4 | Energy, lighting only (15,000 kJ/kg) | 13.5 kWh/day | H1 x 15000 / 3600 / 365 |
| H5 | 400 W panels to cover lighting only (annual mean) | 10.6 | H4 / P2 |
| L1 | 13.88 kWh/day load, annual | 5066 kWh/yr | 13.88 x 365 |
| L2 | Tariff implied by "$2,000/yr" | $0.395/kWh | 2000 / L1 |
| L3 | Same load at Ontario Tier 1 | $608/yr | L1 x 0.120 |
<!-- END GENERATED -->

Notes:
- **L1–L3.** The 13.88 kWh/day load and the $200/month household spend come from her v1/v2 drafts and have no source. L2 shows that "$2,000/yr" implies $0.395/kWh, about 3.3× the Ontario Tier-1 commodity price.
- **Hardware price.** The $499 Starlink kit price (v2.2 README) is not shown on the pages read, so it is not used.

## 5. Implementation plan (hers, with funding named)

- **Beta.** About 1,000 rural users, Starlink only. Service cost to the funder is $900,000/yr (K4).
- **Growth.** 100K+ households ($90,000,000/yr service, K5), with volunteer installers. Reaching 1M+ ($900,000,000/yr, K6) depends on habitats and food becoming real products. Her v2.0 put SpaceX/xAI network benefits at "$1B+ from 1M users"; that figure is unverified.
- **Prototype.** A Starlink demo first, then a habitat/food mock-up during the beta.
- **Risks.** Supply delays, volunteer burnout and token-market capture. Mitigations: a phased rollout, hard burn caps, and no dilution of household claims.

## 6. Token: legal flag and redesign

CSA Staff Notice 46-308 says an offering may be an investment contract if it involves (1) an investment of money (2) in a common enterprise (3) with an expectation of profit (4) coming significantly from the efforts of others. The notice adds that "most of the offerings of tokens purporting to be utility tokens that we have reviewed to date have involved the distribution of a security" [CSA-46-308]. A capped supply with a burn, redeemable for hardware whose platform is still in development, matches the notice's listed risk indicators. **Redesign to test with counsel:**
- non-transferable service credits
- no secondary market
- not purchasable for money
- no fixed supply cap marketed as scarcity
- redeemable only for listed services already available

This lowers the risk but does not remove it. Treat it as **unresolved**.

## 7. Security and privacy model

- **Assets:** household energy and usage data, token balances, guardian control actions.
- **Adversaries:** a fraudulent minter (fake kWh claims), a compromised guardian operator, a data broker.
- **Requirements:** metered kWh signed at the meter; guardian actions logged publicly and subject to user veto; data sharing opt-in under PIPEDA [PIPEDA]; open-source client.

None of this is implemented. It is a specification to build against.

## 8. Prior work

- **Universal broadband targets.** CRTC 2016-496 sets the 50/10 Mbps unlimited objective [CRTC-2016-496]. CRTC CTMR 2026 reports more than 96% of Canadians with USO-level access and over 80% of the rural population covered [CRTC-2026].
- **Hydroponic energy and yield.** Barbosa et al. 2015 [Barbosa-2015].
- **Commercial TEG modules.** Bi₂Te₃, about 4% at ΔT 270 K [Tecteg].
- **Token regulation in Canada.** CSA 46-308 [CSA-46-308].
- **Terrestrial modular habitat.** CHOAM, the companion repository.

## 9. Limitations

- **P1.** Prices are Canadian, read on one date. Other countries differ.
- **P2.** The PV figures use NRCan's Toronto municipal model, not a site simulation. Rural sites farther north have lower December yields.
- **P3.** The TEG analysis uses one datasheet module and assumes the rated ΔT is held for 6 h/day. Real collectors and stoves hold lower and varying ΔT.
- **P4.** The hydroponics model is an Arizona greenhouse. Canadian heating loads would raise total energy, while daylight greenhouses would cut lighting energy.
- **P5.** The Starlink power draw and the $499 hardware price were not verified on a primary page.
- **P6.** Methane backup, habitat modules, guardian software and compute revenue have no quantitative evidence here.
- **P7.** The legal analysis is not legal advice.

## 10. Conclusion

Stellar Network keeps Ashlynn's path: free kit → earned expansions → habitat and food, worldwide. The evidence supports starting with the free kit (funded at $900/household-yr) and PV. TEGs and the token, as designed, are not feasible. Habitat and food are long-term goals with the energy bills shown above. Pilot it, test the numbers, or fork the paper. Contact Ashlynn through a GitHub issue on [Dastille/Stellar-Network](https://github.com/Dastille/Stellar-Network).

## References

- [Barbosa-2015] Barbosa G.L. et al., "Comparison of Land, Water, and Energy Requirements of Lettuce Grown Using Hydroponic vs. Conventional Agricultural Methods", IJERPH 12(6):6879–6891, 2015. https://doi.org/10.3390/ijerph120606879
- [CRTC-2016-496] Telecom Regulatory Policy CRTC 2016-496. https://crtc.gc.ca/eng/archive/2016/2016-496.htm
- [CRTC-2026] CRTC, Canadian Telecommunications Market Report 2026, Executive summary and §4.4. https://crtc.gc.ca/eng/publications/reports/PolicyMonitoring/2026/ctmr.htm
- [CSA-46-308] CSA Staff Notice 46-308, "Securities Law Implications for Offerings of Tokens", 2018-06-11. https://www.osc.ca/en/securities-law/instruments-rules-policies/4/46-308/csa-staff-notice-46-308-securities-law-implications-offerings-tokens
- [NRCan-PV] NRCan, Photovoltaic potential and solar resource maps of Canada, municipal CSVs (municip_potentiel-potential.csv; municip_kWh.csv). https://open.canada.ca/data/en/dataset/8b434ac7-aedb-4698-90df-ba77424a551f
- [OEB-NM] Ontario Energy Board, Net metering. https://oeb.ca/consumer-information-and-protection/net-metering
- [OEB-RPP] Ontario Energy Board, Electricity rates (2025-11-01). https://oeb.ca/consumer-information-and-protection/electricity-rates
- [PIPEDA] https://laws-lois.justice.gc.ca/eng/acts/P-8.6/
- [Starlink-CA] https://www.starlink.com/ca/service-plans and https://www.starlink.com/ca/residential, read 2026-10-18 02:10 EDT (headless Chromium).
- [Tecteg] TEG1-12611-6.0 product data. https://tecteg.com/product/teg1-12611-6-0/

## Appendix A. Change log of ideas (v2.0 → v3.0)

- **Restored (dropped or demoted by the August 2026 rewrites):**
  - free kit as the entry point
  - token
  - TEGs
  - methane backup
  - hydroponic food
  - Starship Mars habitat framing
  - global scope
  - the SpaceX/xAI pitch
  - the whitepaper as the source of truth (the README's "treat it as history" removed)
- **Flagged with evidence:**
  - TEG output (T1–T4)
  - token legality (section 6)
  - "$50–100/month savings" (P4–P5)
  - "self-regulation avoids red tape" (section 2)
  - "$1B+" network benefit (unverified)
- **Unverified, kept as her figures:**
  - $200/month household spend
  - 13.88 kWh/day
  - $2,000/yr
  - 1.5M households lacking access (replaced in the text by CRTC coverage data)
  - 20–30% emissions cut
  - $600/yr family saving
- **Removed:** the title words "Empowering … xAI-Driven Innovation" (marketing language). They are quoted above.
