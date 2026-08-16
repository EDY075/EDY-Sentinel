# ADR 0002: Real data and partial results

Status: accepted.

Collectors return real values or explicit absence. Component failures are attached as collection issues so one unavailable source, such as WMI, does not erase healthy host and network data. Operational mocks and invented defaults are prohibited.
