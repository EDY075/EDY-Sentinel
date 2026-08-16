use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Versioned contract for future explainable rules.
///
/// Sprint 2 hardening defines and validates this data shape only. It does not
/// register, execute, or assign operational severity from rule definitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RuleDefinition {
    pub rule_id: String,
    pub version: u32,
    pub name: String,
    pub description: String,
    pub category: String,
    pub enabled: bool,
    pub conditions: Vec<RuleCondition>,
    pub required_evidence: Vec<EvidenceRequirement>,
    pub severity_policy: RulePolicy,
    pub confidence_policy: RulePolicy,
    pub remediation_guidance: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RuleCondition {
    pub fact_type: String,
    pub field: String,
    pub operator: RuleOperator,
    pub expected: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceRequirement {
    pub fact_type: String,
    pub minimum_count: u32,
    pub correlation_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RulePolicy {
    pub policy_id: String,
    pub version: u32,
    pub description: String,
    pub parameters: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuleOperator {
    Equals,
    NotEquals,
    Exists,
    In,
    AtLeast,
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
            return Err("Rules require conditions and evidence requirements".into());
        }
        if self.required_evidence.iter().any(|requirement| {
            requirement.fact_type.trim().is_empty() || requirement.minimum_count == 0
        }) {
            return Err(
                "Evidence requirements must name a fact and require at least one item".into(),
            );
        }
        if self.severity_policy.version == 0
            || self.confidence_policy.version == 0
            || self.severity_policy.policy_id.trim().is_empty()
            || self.confidence_policy.policy_id.trim().is_empty()
        {
            return Err("Rule policies must be identified and versioned".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{EvidenceRequirement, RuleCondition, RuleDefinition, RuleOperator, RulePolicy};
    use serde_json::json;
    use std::collections::BTreeMap;

    fn definition() -> RuleDefinition {
        RuleDefinition {
            rule_id: "EDY-CONTRACT-001".into(),
            version: 1,
            name: "Contract fixture".into(),
            description: "Serialization fixture; not an operational detection rule".into(),
            category: "contract_test".into(),
            enabled: false,
            conditions: vec![RuleCondition {
                fact_type: "factual_event".into(),
                field: "eventType".into(),
                operator: RuleOperator::Equals,
                expected: json!("fixture"),
            }],
            required_evidence: vec![EvidenceRequirement {
                fact_type: "factual_event".into(),
                minimum_count: 2,
                correlation_fields: vec!["entityKey".into()],
            }],
            severity_policy: RulePolicy {
                policy_id: "corroborated-facts-only".into(),
                version: 1,
                description: "Novelty alone cannot assign severity".into(),
                parameters: BTreeMap::new(),
            },
            confidence_policy: RulePolicy {
                policy_id: "evidence-correlation-quality".into(),
                version: 1,
                description: "Confidence measures evidence quality, not malware probability".into(),
                parameters: BTreeMap::new(),
            },
            remediation_guidance: vec!["Review the factual evidence".into()],
        }
    }

    #[test]
    fn rule_definition_round_trips_with_versions_intact() {
        let value = definition();
        value.validate().expect("valid contract");
        let serialized = serde_json::to_string(&value).expect("serialize contract");
        let restored: RuleDefinition = serde_json::from_str(&serialized).expect("restore contract");
        assert_eq!(restored, value);
        assert!(serialized.contains("\"version\":1"));
        assert!(serialized.contains("corroborated-facts-only"));
    }

    #[test]
    fn rule_definition_rejects_unversioned_or_evidence_free_contracts() {
        let mut value = definition();
        value.version = 0;
        assert!(value.validate().is_err());
        value.version = 1;
        value.required_evidence.clear();
        assert!(value.validate().is_err());
    }
}
