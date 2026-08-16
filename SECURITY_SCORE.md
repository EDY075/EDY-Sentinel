# Security Score v1

## Meaning

Security Score is the observed endpoint posture under current EDY Sentinel coverage. It is not a
percentage of security, a breach probability, or a promise that the device is safe. Formula v1
uses only current rule-produced Detections, their severity/confidence, collector coverage, and
behavioral-baseline readiness.

## Availability and coverage

A number is available only when all of these are true:

- an active behavioral baseline is `Ready`;
- at least one Detection rule is enabled;
- coverage has been measured for `system`, `processes`, `connections`, and `services`;
- none of those required components is failed, unavailable, or paused.

Missing baseline, Learning/Stale/Error, no enabled rules, absent components, or failed coverage
returns `Unavailable` and no number. A degraded component returns `Limited` with an explicit
reason; the numeric calculation is still shown as limited coverage rather than false precision.
Restricted metadata is a coverage fact, not an automatic penalty or Detection.

## Formula version 1

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
`correlation_key` contribute only the largest penalty, preventing one factual cluster from being
counted repeatedly. The sum is capped at 100 and the final score is `100 - capped penalty`.
Occurrence count does not multiply a penalty.

Labels are presentation only:

- 90–100: Excellent
- 75–89: Good
- 50–74: Attention
- 0–49: Elevated Risk

No current v1 rule emits High or Critical. Those enum values and formula weights reserve a
versioned future contract; they are not forced into the current distribution.

## Detection workflow policy

- `New`, `Investigating`, and `Acknowledged` contribute while `condition_active = true`.
- `Acknowledged` means reviewed, not resolved.
- `Resolved` does not contribute to the current score and remains in history.
- `Ignored` does not contribute to the current score and remains in history.
- An inactive condition does not contribute, regardless of workflow status.
- A disabled rule does not contribute while disabled.
- A resolved condition that reappears is reopened by the Detection Engine and contributes again.

## Breakdown and history

The UI exposes base 100, each selected correlation group, severity, confidence, exact penalty,
coverage components, final result, calculation time, and formula version. There are no hidden
inputs.

Snapshots are immutable and persist:

- calculated timestamp and optional numeric score;
- availability state and reason;
- baseline ID;
- formula version;
- active Detection count and highest severity;
- full coverage and breakdown JSON;
- stable input fingerprint and calculation duration;
- schema version.

Identical inputs are deduplicated, with a 15-minute heartbeat snapshot. Retention is 365 days;
future downsampling must retain formula/version meaning.

## Validation evidence

The controlled native smoke produced a Medium/High-confidence correlated Detection and changed
the score from 100 to 92. Resolving it returned the current score to 100. The real SQLite ledger
also preserved a Low/High-confidence state at 97. Fifteen captured formula-v1 snapshots reported
calculation duration below the one-millisecond storage resolution (`0 ms`).

## Limitations

- The score covers only the six local Sprint 2B rules and available Windows collectors.
- It has no external reputation, vulnerability, threat-intelligence, geolocation, or malware
  model input.
- A 100 means no current penalty under measured coverage, not “secure”.
- A short development baseline is unsuitable for production calibration.
- Formula changes require a new `formula_version`; old snapshots must not be reinterpreted.
