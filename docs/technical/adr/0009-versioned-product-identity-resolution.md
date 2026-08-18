# ADR 0009: Versioned conservative product identity resolution

## Status

Accepted for Sprint 3 Part 2.2B.

## Context

Matching Engine v1 could safely evaluate CPE ranges and NVD configuration trees, but exact
normalized registry labels covered too few installed products. Broad fuzzy aliases would improve
recall at the cost of security-significant false positives, especially across vendors, product
editions, launchers, runtimes, and package/build versions.

## Decision

Introduce Product Identity Resolver v1 as a separate, versioned boundary before the unchanged
Matching Engine v1:

- preserve original registry facts and normalize only comparison keys;
- permit exact canonical lookup, a small reviewed alias registry, and reviewed product-specific
  extractors with exact publisher predicates;
- keep vendor and product distinct and reject wrong vendors, similar names, substrings, and
  Unicode-confusable labels;
- preserve product-specific CPE dimensions needed for applicability, including Oracle Java's
  `updateNNN` field;
- return explicit resolved, ambiguous, or unresolved status and a stable reason code;
- persist resolver version, method, confidence, canonical identity/CPE, and immutable provenance;
- leave helpers, launchers, redistributables, suites, and unsupported version grammars unresolved;
- require every unobserved CPE dimension to be wildcard before a criterion can confirm;
- allow only narrow, reviewed product-applicability rejections when authoritative product-family
  evidence conflicts with an NVD-derived candidate.

No CVE description text participates in identity or applicability. CISA KEV remains enrichment
after an association and cannot promote a finding. Product Identity results do not feed Detection
Engine, rule severity, remediation, or Security Score.

## Consequences

Coverage increases for reviewed installed products while false-positive controls remain
fail-closed. Adding coverage now requires a versioned alias/extractor change plus negative fixtures
for vendor, similar/different product, missing/invalid version, casing, and confusable Unicode.
Resolver evidence is queryable in historical evaluations and visible in the Inventory detail UI.

The approach intentionally leaves many products unresolved. It also increases indexed candidate
work for products newly mapped to canonical CPE identities; performance must be measured against the
real local cache whenever the registry expands.
