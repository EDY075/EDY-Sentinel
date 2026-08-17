# Security Score v1 and v2

## Meaning

Security Score is the observed endpoint posture under the coverage currently available to EDY
Sentinel. It is not a percentage of security, a breach probability, or a promise that the device
is safe. A score of 100 means only that no current eligible penalty was observed.

Formula v1 uses active rule-produced Detections. Formula v2 preserves that component and adds a
bounded, explainable Vulnerability Intelligence component derived only from current
`Confirmed + High confidence` software-to-CVE matches.

## Availability and coverage

A number requires a Ready behavioral baseline, at least one enabled Detection rule, and measured
coverage for system, processes, connections, and services. Missing required inputs or failed
required collectors return `Unavailable` with an explicit reason.

In formula v2, vulnerability coverage is reported independently:

- `updating` or `unavailable` qualifies the numeric score as limited coverage because the current
  vulnerability component may be incomplete;
- `limited` after evaluation means eligible software identities remain unresolved or ambiguous,
  but does not by itself hide the numeric score or create a penalty;
- `complete` means every eligible current identity is resolved and evaluated;
- `not_applicable` means there are no eligible products.

Restricted metadata, unresolved identities, ambiguous identities, `Possible` CVEs, and software
classified as not mappable are coverage facts. They never create a vulnerability penalty.

## Formula version 1: Detections

The base is 100. Each active Detection maps to a base penalty and a confidence factor:

| Severity | Base penalty |
| --- | ---: |
| Informational | 0 |
| Low | 3 |
| Medium | 8 |
| High | 18 |
| Critical | 35 |

| Confidence | Factor |
| --- | ---: |
| Low | 0.50 |
| Medium | 0.75 |
| High | 1.00 |

For each Detection: `round(base penalty × confidence factor)`. Detections sharing a
`correlation_key` contribute only the largest penalty. The Detection contribution is capped at
100. Occurrence count does not multiply a penalty.

Workflow policy:

- `New`, `Investigating`, and `Acknowledged` contribute while `condition_active = true`;
- `Resolved`, `Ignored`, inactive conditions, and disabled rules do not contribute;
- a resolved condition that reappears is reopened by the Detection Engine and contributes again.

## Formula version 2: Detections plus Vulnerabilities

Formula v2 leaves formula-v1 Detection calculation unchanged and adds Product Vulnerability Risk.
Only active software with a current completed evaluation, a non-ambiguous canonical identity, and
a `Confirmed + High confidence` match is eligible. `Possible`, `Unresolved`, `Not affected`, stale
fingerprints, and non-High confidence matches have score impact zero.

For each canonical product, unique confirmed CVEs are reduced to broad NVD CVSS weights:

| NVD CVSS band | Weight |
| --- | ---: |
| Unrated or Low | 0.5 |
| Medium | 1.0 |
| High | 2.0 |
| Critical | 3.0 |

With weights sorted descending:

```text
severity_anchor = highest weight
marginal_breadth = min(2.00, 0.25 × sum(remaining weights))
kev_boost = 3.00 when at least one already-confirmed CVE is in CISA KEV, otherwise 0
product_impact = min(7, floor(severity_anchor + marginal_breadth + kev_boost + 0.5))

D = formula-v1 Detection contribution
V = min(18, sum(product_impact))
Security Score v2 = 100 - min(100, D + V)
```

KEV is enrichment and prioritization after matching. A KEV record never confirms a software-to-CVE
relationship. CVEs and duplicate Registry records are deduplicated inside the current canonical
product context; historical evaluations never multiply current impact.

The validated current calculation is:

```text
Base                                  100
Detections                              0
Vulnerabilities                       -14
  Oracle JRE 8u401                     -6  (6 Confirmed, 1 KEV)
  Oracle VirtualBox 7.2.12             -4  (15 Confirmed)
  Python 3.12.10                       -4  (10 Confirmed, 1 Possible at impact 0)
Final                                  86
Formula                                v2
```

## Labels and presentation

Labels are presentation only:

- 90–100: Excellent
- 75–89: Good
- 50–74: Attention
- 0–49: Elevated Risk

The UI always separates Detection and Vulnerability contributions, software identity coverage,
module coverage, formula version, and calculation time. Software counts are never presented as CVE
counts. A limited-but-numeric state keeps the value visible with a clear coverage qualifier.

## Breakdown and history

The v2 breakdown exposes base 100, Detection contribution, Vulnerability contribution, final
score, canonical products, confirmed/possible counts, highest CVSS, KEV context, product impact,
coverage counts, evidence-engine versions, calculation time, and formula version. Product actions
navigate to the existing Inventory and CVE evidence surfaces.

Snapshots are immutable and persist:

- calculated timestamp and optional numeric score;
- availability state and reason;
- baseline ID and formula version;
- active Detection count and highest Detection severity;
- vulnerability contribution and per-product risk evidence for v2;
- full coverage and breakdown JSON;
- stable input fingerprint, duration, and schema version.

Identical inputs are deduplicated with a 15-minute heartbeat. Retention is 365 days. Formula-v1
snapshots remain formula v1 and are never recalculated or reinterpreted as v2.

## Validation evidence

- Formula-v1 controlled lifecycle: 100 → 92 → 100 around an active correlated Detection.
- Formula-v2 real dataset: 31 Confirmed CVEs, 1 Possible at impact zero, 1 Confirmed KEV, product
  impacts 6/4/4, Detection impact 0, Vulnerability impact 14, final score 86.
- Real Tauri release lifecycle: 81 pending evaluations qualified the score as limited; after the
  queue completed, pending became 0 and score state returned to Good at 86. Coverage remained
  Limited because 47 of 66 eligible software records are unresolved, without a fabricated penalty.
- Automated Rust tests preserve v1/v2 snapshot history and cover removal, version change,
  Not affected, stale fingerprint, Possible-only, duplicate, cap, KEV, and coverage transitions.

## Limitations

- The Detection component covers the six local Sprint 2B rules.
- Vulnerability coverage is currently a conservative per-software-record proxy, not yet a fully
  deduplicated product denominator.
- The model does not infer runtime exposure, exploit probability, remediation state, or breach.
- A short development baseline is unsuitable for production calibration.
- Any formula-weight or cap change requires a new `formula_version`; old snapshots must retain
  their original interpretation.
