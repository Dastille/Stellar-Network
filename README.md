# Stellar Network

Ideas for self-sufficiency and megacorporation-era growth (Ashlynn, 2025).

**The vision (Ashlynn's v2.0, kept as the thesis):**
1. A **free, fully installed Starlink kit** for rural and self-sufficiency-seeking households worldwide.
2. Households **earn tokens** by sharing energy, data and compute.
3. Tokens pay for expansions: 400 W solar, used EV batteries, **TEGs** with heat storage, **methane backup**.
4. Long term: **Starship-class habitat modules** and **hydroponic food**.

**The paper is the source of truth:** [whitepaper/whitepaper.md](whitepaper/whitepaper.md) (v3.0). This README summarizes it. Claim statuses are in [CLAIMS.md](CLAIMS.md).

This is an independent proposal. It is not a SpaceX, Starlink or xAI product and is not endorsed by them.

## What the numbers say (from `model/`, asserted by `cargo test`)

- **Free kit cost.** Starlink Residential 100 in Canada is **CA$75**/mo (starlink.com/ca, read 2026-10-18). Free service therefore costs a funder **$900** per household-year, or **$900,000,000** per year at 1M households.
- **Solar.** A 400 W panel yields **1.27 kWh/day** in Toronto, worth **$4.65/mo** at Ontario Tier 1.
- **TEGs.** Her 1.8 kWh/day from 5 × 60 W TEGs needs about **45 kWh(th)/day** of 300 °C heat at the datasheet's 4.0% efficiency. **0.56 kWp** of PV gives the same energy. TEGs stay in the paper as a stove co-generator idea, not as primary supply.
- **Hydroponic food.** 50 lb/week of greens needs about **28.8 m2** of growing area and **13.5 kWh/day** for lighting alone.
- **Token.** An earn-and-redeem token is likely a security under CSA Staff Notice 46-308. The paper proposes a redesign and marks the question unresolved.
- **Habitat and methane.** No Starship habitat product exists. Methane backup is unverified. Both are kept as goals, with the habitat path linked to [CHOAM](https://github.com/Dastille/CHOAM-initiative).

## Reproduce

```sh
cargo test                        # asserts the paper's table == model output
cargo run -q -p stellar-model     # prints the table embedded in the paper
```

## Status

- Whitepaper v3.0 (restored vision, sourced, with limitations)
- Concept art (`grok_image_*.jpg`) conveys mood, not engineering

License: [AGPL-3.0](LICENSE)

— Ashlynn / [Dastille](https://github.com/Dastille)
