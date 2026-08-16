use crate::models::{DetectionConfidence, DetectionSeverity};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::OnceLock};

/// Versioned, declarative metadata for an explainable detection rule.
///
/// This module intentionally contains no generic evaluator. Conditions are
/// allowlisted metadata consumed by the Rust detection engine, never SQL,
/// scripts, or user-editable expressions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuleDefinition {
    pub rule_id: String,
    pub version: u32,
    pub name: String,
    pub description: String,
    pub category: String,
    pub enabled: bool,
    pub references: Vec<String>,
    /// Every positive condition is required.
    pub conditions: Vec<RuleCondition>,
    /// Any matching exclusion prevents the rule from firing.
    pub exclusions: Vec<RuleCondition>,
    pub required_evidence: Vec<EvidenceRequirement>,
    pub correlation_window_seconds: u32,
    pub default_severity: DetectionSeverity,
    pub default_confidence: DetectionConfidence,
    pub severity_policy: RulePolicy,
    pub confidence_policy: RulePolicy,
    pub precedence: u16,
    pub supersedes: Vec<String>,
    pub score_group_template: String,
    pub false_positive_considerations: Vec<String>,
    pub remediation_guidance: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuleCondition {
    pub fact_type: String,
    pub field: String,
    pub operator: RuleOperator,
    pub expected: ConditionValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceRequirement {
    pub evidence_id: String,
    pub fact_type: String,
    pub accepted_event_types: Vec<String>,
    pub minimum_count: u32,
    pub correlation_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RulePolicy {
    pub policy_id: String,
    pub version: u32,
    pub description: String,
    pub parameters: BTreeMap<String, ConditionValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ConditionValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Strings(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuleOperator {
    Equals,
    NotEquals,
    Exists,
    In,
    AtLeast,
    ClassifiedAs,
}

impl RuleDefinition {
    pub fn validate(&self) -> Result<(), String> {
        if self.rule_id.trim().is_empty() || self.version == 0 {
            return Err("Rule identity and version are required".into());
        }
        if self.name.trim().is_empty()
            || self.description.trim().is_empty()
            || self.category.trim().is_empty()
        {
            return Err("Rule metadata is incomplete".into());
        }
        if self.conditions.is_empty() || self.required_evidence.is_empty() {
            return Err("Rules require positive conditions and evidence".into());
        }
        if self.correlation_window_seconds == 0 || self.correlation_window_seconds > 3_600 {
            return Err("Rule correlation window must be between 1 and 3600 seconds".into());
        }
        if self
            .conditions
            .iter()
            .chain(&self.exclusions)
            .any(|condition| {
                condition.fact_type.trim().is_empty() || condition.field.trim().is_empty()
            })
        {
            return Err("Rule conditions must identify a fact and field".into());
        }
        if self.required_evidence.iter().any(|requirement| {
            requirement.evidence_id.trim().is_empty()
                || requirement.fact_type.trim().is_empty()
                || requirement.accepted_event_types.is_empty()
                || requirement.minimum_count == 0
                || requirement.correlation_fields.is_empty()
        }) {
            return Err("Evidence requirements must be identified and correlatable".into());
        }
        if self.severity_policy.version == 0
            || self.confidence_policy.version == 0
            || self.severity_policy.policy_id.trim().is_empty()
            || self.confidence_policy.policy_id.trim().is_empty()
        {
            return Err("Rule policies must be identified and versioned".into());
        }
        if self.precedence == 0
            || self.score_group_template.trim().is_empty()
            || !self.score_group_template.contains('{')
            || self
                .supersedes
                .iter()
                .any(|rule_id| rule_id == &self.rule_id)
        {
            return Err("Rule precedence and score grouping are invalid".into());
        }
        if self.references.is_empty()
            || self.false_positive_considerations.is_empty()
            || self.remediation_guidance.is_empty()
        {
            return Err("Rule explanation metadata is incomplete".into());
        }
        Ok(())
    }
}

static RULE_REGISTRY: OnceLock<Vec<RuleDefinition>> = OnceLock::new();

/// Returns the immutable built-in Sprint 2B rule registry.
pub fn registry() -> &'static [RuleDefinition] {
    RULE_REGISTRY.get_or_init(built_in_rules).as_slice()
}

/// Looks up the exact definition that produced a detection.
pub fn lookup(rule_id: &str, version: u32) -> Option<&'static RuleDefinition> {
    registry()
        .iter()
        .find(|rule| rule.rule_id == rule_id && rule.version == version)
}

fn built_in_rules() -> Vec<RuleDefinition> {
    vec![
        process_temp_unsigned(),
        process_contextual_parent_child(),
        network_new_unsigned_outbound(),
        service_new_privileged_autostart(),
        service_correlated_reconfiguration(),
        network_coordinated_configuration_change(),
    ]
}

fn process_temp_unsigned() -> RuleDefinition {
    rule(
        "EDY-PROC-001",
        "New unsigned executable in a temporary user-writable path",
        "A new executable was actually launched from a temporary user-writable path and Windows reported it as unsigned.",
        "process_execution",
        vec![
            condition("baseline", "status", RuleOperator::Equals, text("ready")),
            condition("executable_first_seen", "evidence.signatureStatus", RuleOperator::Equals, text("unsigned")),
            condition("executable_first_seen", "evidence.path", RuleOperator::ClassifiedAs, text("user_writable_temp")),
            condition("correlation", "sameNormalizedExecutablePath", RuleOperator::Equals, boolean(true)),
        ],
        common_executable_exclusions(),
        vec![
            evidence("new_executable", &["executable_first_seen"], &["evidence.path"]),
            evidence("executed_process", &["process_first_seen"], &["evidence.path"]),
        ],
        60,
        DetectionSeverity::Low,
        DetectionConfidence::High,
        100,
        vec![],
        "executable:{entityKey}",
        &["security_event:executable_first_seen@v1", "security_event:process_first_seen@v1"],
        &[
            "A legitimate installer or updater may extract an unsigned helper into Temp.",
            "A development tool may execute a freshly built unsigned binary from Temp.",
        ],
        &[
            "Verify the executable origin and the application that launched it.",
            "Review the file signature and recent installation activity without deleting the file automatically.",
        ],
    )
}

fn process_contextual_parent_child() -> RuleDefinition {
    let mut exclusions = common_executable_exclusions();
    exclusions.push(condition(
        "correlation",
        "qualifyingOutboundDestinationPresent",
        RuleOperator::Equals,
        boolean(true),
    ));
    rule(
        "EDY-PROC-002",
        "New unsigned child execution with a known signed parent",
        "A baseline-known signed parent launched a new unsigned child from a temporary user-writable path through a previously unseen relationship.",
        "process_execution",
        vec![
            condition("baseline", "status", RuleOperator::Equals, text("ready")),
            condition("executable_first_seen", "evidence.signatureStatus", RuleOperator::Equals, text("unsigned")),
            condition("executable_first_seen", "evidence.path", RuleOperator::ClassifiedAs, text("user_writable_temp")),
            condition("baseline_parent", "signatureStatus", RuleOperator::Equals, text("signed")),
            condition("correlation", "sameNormalizedChildPath", RuleOperator::Equals, boolean(true)),
        ],
        exclusions,
        vec![
            evidence("new_executable", &["executable_first_seen"], &["evidence.path"]),
            evidence("executed_process", &["process_first_seen"], &["evidence.path"]),
            evidence("new_parent_child", &["parent_child_first_seen"], &["evidence.childPath", "evidence.parentPath"]),
        ],
        60,
        DetectionSeverity::Medium,
        DetectionConfidence::High,
        200,
        vec!["EDY-PROC-001"],
        "executable:{entityKey}",
        &["security_event:parent_child_first_seen@v1", "baseline_executables@v1"],
        &[
            "Signed installers and browsers may launch unsigned temporary helpers.",
            "Enterprise deployment tools may create a new parent-child relationship during updates.",
        ],
        &[
            "Verify whether the parent application was performing an expected install or update.",
            "Compare the child path, signer state, and creation context with recent user activity.",
        ],
    )
}

fn network_new_unsigned_outbound() -> RuleDefinition {
    rule(
        "EDY-NET-001",
        "New outbound activity from a new unsigned temporary executable",
        "A newly executed unsigned binary in a temporary user-writable path initiated first-seen outbound activity with an unambiguous process association.",
        "network_activity",
        vec![
            condition("baseline", "status", RuleOperator::Equals, text("ready")),
            condition("executable_first_seen", "evidence.signatureStatus", RuleOperator::Equals, text("unsigned")),
            condition("executable_first_seen", "evidence.path", RuleOperator::ClassifiedAs, text("user_writable_temp")),
            condition("destination_first_seen", "evidence.association", RuleOperator::Equals, text("associated")),
            condition("destination_first_seen", "evidence.remoteIp", RuleOperator::ClassifiedAs, text("unicast_remote")),
            condition("correlation", "uniqueProcessNameMatch", RuleOperator::Equals, boolean(true)),
        ],
        vec![
            condition("baseline", "status", RuleOperator::In, texts(&["not_initialized", "learning", "stale", "error"])),
            condition("executable_first_seen", "evidence.signatureStatus", RuleOperator::In, texts(&["unknown", "restricted", "signed"])),
            condition("executable_first_seen", "evidence.path", RuleOperator::ClassifiedAs, text("missing_ambiguous_or_not_user_writable_temp")),
            condition("baseline_executable", "sameNormalizedPathExists", RuleOperator::Equals, boolean(true)),
            condition("destination_first_seen", "evidence.association", RuleOperator::In, texts(&["unresolved", "recently_exited", "system_kernel", "not_applicable"])),
            condition("destination_first_seen", "evidence.remoteIp", RuleOperator::ClassifiedAs, text("loopback_unspecified_or_multicast")),
            condition("correlation", "ambiguousProcessNameMatch", RuleOperator::Equals, boolean(true)),
        ],
        vec![
            evidence("new_executable", &["executable_first_seen"], &["evidence.path", "evidence.process"]),
            evidence("executed_process", &["process_first_seen"], &["evidence.path", "evidence.process"]),
            evidence("new_destination", &["destination_first_seen"], &["evidence.process"]),
        ],
        60,
        DetectionSeverity::Medium,
        DetectionConfidence::Medium,
        300,
        vec!["EDY-PROC-002", "EDY-PROC-001"],
        "executable:{entityKey}",
        &["security_event:destination_first_seen@v1", "collector:connections@v1"],
        &[
            "Legitimate installers, updaters, and development tools may download content from a temporary helper.",
            "A private-network destination may be an expected local service even when it is new to the baseline.",
        ],
        &[
            "Review the executable origin, parent process, remote address, port, and recent installation activity.",
            "Confirm whether the destination is expected for the application; do not block it automatically.",
        ],
    )
}

fn service_new_privileged_autostart() -> RuleDefinition {
    rule(
        "EDY-SVC-001",
        "New privileged automatic service from a user-writable path",
        "A newly observed running service uses an automatic startup mode, a privileged system account, and a binary under a user-writable location.",
        "service_persistence",
        vec![
            condition("baseline", "status", RuleOperator::Equals, text("ready")),
            condition("service_first_seen", "evidence.status", RuleOperator::Equals, text("Running")),
            condition("service_first_seen", "evidence.startupType", RuleOperator::In, texts(&["Automatic", "Automatic Delayed"])),
            condition("service_first_seen", "evidence.account", RuleOperator::In, texts(&["LocalSystem", "NT AUTHORITY\\SYSTEM"])),
            condition("service_first_seen", "evidence.binaryPath", RuleOperator::ClassifiedAs, text("user_writable")),
        ],
        vec![
            condition("baseline", "status", RuleOperator::In, texts(&["not_initialized", "learning", "stale", "error"])),
            condition("collector.services", "status", RuleOperator::Equals, text("failed")),
            condition("service_first_seen", "evidence.startupType", RuleOperator::In, texts(&["Unavailable", "Manual", "Disabled"])),
            condition("service_first_seen", "evidence.binaryPath", RuleOperator::ClassifiedAs, text("missing_or_ambiguous")),
        ],
        vec![evidence("new_service", &["service_first_seen"], &["evidence.serviceName"])],
        30,
        DetectionSeverity::Medium,
        DetectionConfidence::High,
        200,
        vec![],
        "service:{entityKey}",
        &["security_event:service_first_seen@v1", "collector:services@v1"],
        &[
            "A legitimate vendor updater may install an automatic service under a system account.",
            "Enterprise management software may legitimately run a service binary from a managed user-writable location.",
        ],
        &[
            "Verify the service origin, binary path, account, and recent software installation.",
            "Review the binary signature separately because service facts do not currently include signer metadata.",
        ],
    )
}

fn service_correlated_reconfiguration() -> RuleDefinition {
    rule(
        "EDY-SVC-002",
        "Correlated persistent service reconfiguration",
        "A known service changed its binary path together with its startup type or account in the same collection window.",
        "service_persistence",
        vec![
            condition("baseline", "status", RuleOperator::Equals, text("ready")),
            condition("correlation", "sameServiceName", RuleOperator::Equals, boolean(true)),
            condition("correlation", "distinctChangedFields", RuleOperator::AtLeast, integer(2)),
            condition("correlation", "binaryPathChanged", RuleOperator::Equals, boolean(true)),
        ],
        vec![
            condition("baseline", "status", RuleOperator::In, texts(&["not_initialized", "learning", "stale", "error"])),
            condition("collector.services", "status", RuleOperator::Equals, text("failed")),
            condition("correlation", "distinctChangedFields", RuleOperator::Equals, integer(1)),
            condition("correlation", "missingAfterValue", RuleOperator::Equals, boolean(true)),
        ],
        vec![
            evidence("binary_change", &["service_binary_changed"], &["evidence.serviceName"]),
            evidence("additional_configuration_change", &["service_startup_changed", "service_account_changed"], &["evidence.serviceName"]),
        ],
        45,
        DetectionSeverity::Medium,
        DetectionConfidence::High,
        200,
        vec![],
        "service:{entityKey}",
        &["security_event:service_binary_changed@v1", "security_event:service_startup_changed@v1", "security_event:service_account_changed@v1"],
        &[
            "A legitimate software upgrade may change a service binary and startup configuration together.",
            "Security and device-management software may rotate service accounts during maintenance.",
        ],
        &[
            "Review the before and after values and correlate them with an approved software change.",
            "Verify the new binary origin and account privileges without modifying the service automatically.",
        ],
    )
}

fn network_coordinated_configuration_change() -> RuleDefinition {
    rule(
        "EDY-NET-002",
        "Persistent coordinated network configuration change",
        "The default gateway and DNS resolver set changed together and remained different from the baseline for at least two observations.",
        "network_configuration",
        vec![
            condition("baseline", "status", RuleOperator::Equals, text("ready")),
            condition("correlation", "sameBaseline", RuleOperator::Equals, boolean(true)),
            condition("correlation", "minimumObservationCount", RuleOperator::AtLeast, integer(2)),
            condition("correlation", "gatewayAndDnsChanged", RuleOperator::Equals, boolean(true)),
        ],
        vec![
            condition("baseline", "status", RuleOperator::In, texts(&["not_initialized", "learning", "stale", "error"])),
            condition("collector.system", "status", RuleOperator::In, texts(&["degraded", "failed"])),
            condition("correlation", "revertedBeforeSecondObservation", RuleOperator::Equals, boolean(true)),
            condition("correlation", "missingBeforeOrAfter", RuleOperator::Equals, boolean(true)),
        ],
        vec![
            evidence("gateway_change", &["gateway_changed"], &["baselineId"]),
            evidence("dns_change", &["dns_changed"], &["baselineId"]),
        ],
        30,
        DetectionSeverity::Low,
        DetectionConfidence::High,
        100,
        vec![],
        "network_configuration:{baselineId}",
        &["security_event:gateway_changed@v1", "security_event:dns_changed@v1", "security_event:primary_route_changed@v1"],
        &[
            "VPN activation, Wi-Fi roaming, DHCP renewal, or a router change may legitimately alter gateway and DNS together.",
            "A managed corporate network may intentionally replace resolvers and routes.",
        ],
        &[
            "Confirm whether a VPN, Wi-Fi, DHCP, or router change was expected.",
            "Compare the before and after gateway, resolver, interface, and route values.",
        ],
    )
}

#[allow(clippy::too_many_arguments)]
fn rule(
    rule_id: &str,
    name: &str,
    description: &str,
    category: &str,
    conditions: Vec<RuleCondition>,
    exclusions: Vec<RuleCondition>,
    required_evidence: Vec<EvidenceRequirement>,
    correlation_window_seconds: u32,
    default_severity: DetectionSeverity,
    default_confidence: DetectionConfidence,
    precedence: u16,
    supersedes: Vec<&str>,
    score_group_template: &str,
    references: &[&str],
    false_positive_considerations: &[&str],
    remediation_guidance: &[&str],
) -> RuleDefinition {
    RuleDefinition {
        rule_id: rule_id.into(),
        version: 1,
        name: name.into(),
        description: description.into(),
        category: category.into(),
        enabled: true,
        references: strings(references),
        conditions,
        exclusions,
        required_evidence,
        correlation_window_seconds,
        default_severity,
        default_confidence,
        severity_policy: policy(
            "corroborated-facts-only",
            "Novelty, unsigned state, or location alone cannot assign operational severity.",
            BTreeMap::from([
                ("maximumSeverity".into(), text("medium")),
                ("noveltyAloneTriggers".into(), boolean(false)),
            ]),
        ),
        confidence_policy: policy(
            "evidence-correlation-quality",
            "Confidence measures evidence availability and correlation quality, not malware probability.",
            BTreeMap::from([
                ("missingEvidenceAction".into(), text("do_not_trigger")),
                ("unknownSignatureIsUnsigned".into(), boolean(false)),
            ]),
        ),
        precedence,
        supersedes: strings(&supersedes),
        score_group_template: score_group_template.into(),
        false_positive_considerations: strings(false_positive_considerations),
        remediation_guidance: strings(remediation_guidance),
    }
}

fn common_executable_exclusions() -> Vec<RuleCondition> {
    vec![
        condition(
            "baseline",
            "status",
            RuleOperator::In,
            texts(&["not_initialized", "learning", "stale", "error"]),
        ),
        condition(
            "executable_first_seen",
            "evidence.signatureStatus",
            RuleOperator::In,
            texts(&["unknown", "restricted", "signed"]),
        ),
        condition(
            "executable_first_seen",
            "evidence.path",
            RuleOperator::ClassifiedAs,
            text("missing_ambiguous_or_not_user_writable_temp"),
        ),
        condition(
            "baseline_executable",
            "sameNormalizedPathExists",
            RuleOperator::Equals,
            boolean(true),
        ),
        condition(
            "correlation",
            "ambiguousProcessMatch",
            RuleOperator::Equals,
            boolean(true),
        ),
    ]
}

fn condition(
    fact_type: &str,
    field: &str,
    operator: RuleOperator,
    expected: ConditionValue,
) -> RuleCondition {
    RuleCondition {
        fact_type: fact_type.into(),
        field: field.into(),
        operator,
        expected,
    }
}

fn evidence(
    evidence_id: &str,
    accepted_event_types: &[&str],
    correlation_fields: &[&str],
) -> EvidenceRequirement {
    EvidenceRequirement {
        evidence_id: evidence_id.into(),
        fact_type: "security_event".into(),
        accepted_event_types: strings(accepted_event_types),
        minimum_count: 1,
        correlation_fields: strings(correlation_fields),
    }
}

fn policy(
    policy_id: &str,
    description: &str,
    parameters: BTreeMap<String, ConditionValue>,
) -> RulePolicy {
    RulePolicy {
        policy_id: policy_id.into(),
        version: 1,
        description: description.into(),
        parameters,
    }
}

fn text(value: &str) -> ConditionValue {
    ConditionValue::String(value.into())
}

fn integer(value: i64) -> ConditionValue {
    ConditionValue::Integer(value)
}

fn boolean(value: bool) -> ConditionValue {
    ConditionValue::Boolean(value)
}

fn texts(values: &[&str]) -> ConditionValue {
    ConditionValue::Strings(strings(values))
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).into()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn registry_contains_exactly_six_enabled_versioned_rules() {
        let rules = registry();
        assert_eq!(rules.len(), 6);
        assert_eq!(
            rules
                .iter()
                .map(|rule| rule.rule_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "EDY-PROC-001",
                "EDY-PROC-002",
                "EDY-NET-001",
                "EDY-SVC-001",
                "EDY-SVC-002",
                "EDY-NET-002",
            ]
        );
        assert!(rules.iter().all(|rule| rule.enabled && rule.version == 1));
        assert!(rules
            .iter()
            .all(|rule| rule.default_severity <= DetectionSeverity::Medium));
    }

    #[test]
    fn registry_metadata_is_valid_unique_and_serializable() {
        let mut identities = HashSet::new();
        for rule in registry() {
            rule.validate().expect("built-in rule must be valid");
            assert!(identities.insert((rule.rule_id.clone(), rule.version)));
            let serialized = serde_json::to_string(rule).expect("serialize rule");
            let restored: RuleDefinition = serde_json::from_str(&serialized).expect("restore rule");
            assert_eq!(&restored, rule);
            assert!(serialized.contains("correlationWindowSeconds"));
            assert!(serialized.contains("falsePositiveConsiderations"));
        }
    }

    #[test]
    fn lookup_requires_the_exact_rule_version() {
        assert_eq!(
            lookup("EDY-NET-001", 1).map(|rule| rule.name.as_str()),
            Some("New outbound activity from a new unsigned temporary executable")
        );
        assert!(lookup("EDY-NET-001", 2).is_none());
        assert!(lookup("missing", 1).is_none());
    }

    #[test]
    fn signature_rules_never_treat_unknown_or_restricted_as_unsigned() {
        for rule_id in ["EDY-PROC-001", "EDY-PROC-002", "EDY-NET-001"] {
            let rule = lookup(rule_id, 1).expect("signature rule");
            assert!(rule.conditions.iter().any(|condition| {
                condition.field == "evidence.signatureStatus"
                    && condition.operator == RuleOperator::Equals
                    && condition.expected == text("unsigned")
            }));
            assert!(rule.exclusions.iter().any(|condition| {
                condition.field == "evidence.signatureStatus"
                    && condition.operator == RuleOperator::In
                    && condition.expected == texts(&["unknown", "restricted", "signed"])
            }));
            assert_eq!(
                rule.confidence_policy
                    .parameters
                    .get("unknownSignatureIsUnsigned"),
                Some(&boolean(false))
            );
        }
    }

    #[test]
    fn supersession_targets_exist_and_have_lower_precedence() {
        for rule in registry() {
            for target in &rule.supersedes {
                let target = lookup(target, 1).expect("superseded rule must exist");
                assert!(target.precedence < rule.precedence);
                assert_eq!(target.score_group_template, rule.score_group_template);
            }
        }
    }

    #[test]
    fn validation_rejects_unversioned_or_unexplainable_rules() {
        let mut invalid = registry()[0].clone();
        invalid.version = 0;
        assert!(invalid.validate().is_err());
        invalid.version = 1;
        invalid.false_positive_considerations.clear();
        assert!(invalid.validate().is_err());
        invalid.false_positive_considerations = vec!["fixture".into()];
        invalid.correlation_window_seconds = 0;
        assert!(invalid.validate().is_err());
    }
}
