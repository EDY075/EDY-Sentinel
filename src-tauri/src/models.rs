use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemOverview {
    pub collected_at: String,
    pub source: String,
    pub host: HostInfo,
    pub operating_system: OperatingSystemInfo,
    pub cpu: CpuInfo,
    pub gpus: Vec<GpuInfo>,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
    pub network: NetworkInfo,
    pub collector: CollectorHealth,
    pub issues: Vec<CollectionIssue>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostInfo {
    pub hostname: String,
    pub username: String,
    pub architecture: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatingSystemInfo {
    pub name: String,
    pub edition: Option<String>,
    pub display_version: Option<String>,
    pub build: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuInfo {
    pub model: String,
    pub logical_cores: usize,
    pub physical_cores: Option<usize>,
    pub frequency_mhz: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub name: String,
    pub adapter_ram_bytes: Option<u64>,
    pub driver_version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub removable: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInfo {
    pub primary_interface: Option<String>,
    pub primary_interface_type: Option<String>,
    pub primary_ipv4: Option<String>,
    pub primary_gateway: Option<String>,
    pub primary_route_metric: Option<u32>,
    pub gateways: Vec<String>,
    pub dns_servers: Vec<String>,
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterface {
    pub name: String,
    pub friendly_name: String,
    pub description: String,
    pub interface_type: String,
    pub classification_source: String,
    pub operational_status: String,
    pub ipv4_metric: u32,
    pub primary_route: bool,
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
    pub gateways: Vec<String>,
    pub dns_servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionIssue {
    pub component: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    pub id: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseStatus {
    pub schema_version: i64,
    pub path_kind: String,
    pub writable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareIdentity {
    pub vendor: String,
    pub product: String,
    pub version: String,
    pub architecture: String,
    pub install_scope: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InstalledSoftwareRecord {
    pub software_id: String,
    pub display_name: String,
    pub display_version: Option<String>,
    pub publisher: Option<String>,
    pub install_location: Option<String>,
    pub install_date: Option<String>,
    pub architecture: String,
    pub install_scope: String,
    pub sources: Vec<String>,
    pub registry_identities: Vec<String>,
    pub product_code: Option<String>,
    pub normalized_identity: SoftwareIdentity,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub observation_count: u64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareInventorySnapshot {
    pub items: Vec<InstalledSoftwareRecord>,
    pub collected_at: String,
    pub duration_ms: u64,
    pub raw_entry_count: usize,
    pub source_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VulnerabilityProviderStatus {
    pub provider: String,
    pub status: String,
    pub last_attempt_at: Option<String>,
    pub last_successful_sync_at: Option<String>,
    pub record_count: u64,
    pub records_processed: u64,
    pub pages_processed: u64,
    pub last_successful_page: Option<u64>,
    pub sync_elapsed_ms: Option<u64>,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VulnerabilitySyncInput {
    pub provider: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VulnerabilityEvaluationInput {
    pub software_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareVulnerabilityInput {
    pub software_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareVulnerabilitySummary {
    pub software_id: String,
    pub evaluation_state: String,
    pub confirmed_count: u64,
    pub possible_count: u64,
    pub unresolved_count: u64,
    pub not_affected_count: u64,
    pub highest_cvss: Option<f64>,
    pub kev_count: u64,
    pub last_evaluated_at: Option<String>,
    pub matching_engine_version: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VulnerabilityEvaluationSummary {
    pub software_evaluated: u64,
    pub confirmed: u64,
    pub possible: u64,
    pub unresolved: u64,
    pub not_affected: u64,
    pub confirmed_cves: u64,
    pub kev_cves: u64,
    pub candidate_cves: u64,
    pub duration_ms: u64,
    pub matching_engine_version: u32,
    pub remaining_queued: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VulnerabilityMatchRecord {
    pub match_id: String,
    pub cve_id: String,
    pub match_state: String,
    pub confidence: String,
    pub installed_version: Option<String>,
    pub cpe: String,
    pub affected_range: String,
    pub comparison_result: String,
    pub cvss_score: Option<f64>,
    pub cvss_version: Option<String>,
    pub severity: Option<String>,
    pub description: String,
    pub published_at: String,
    pub last_modified_at: String,
    pub references: Vec<String>,
    pub kev: Option<KevContext>,
    pub evidence: Vec<VulnerabilityEvidenceRecord>,
    pub last_evaluated_at: String,
    pub matching_engine_version: u32,
    pub nvd_source_version: Option<String>,
    pub kev_source_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KevContext {
    pub vulnerability_name: String,
    pub date_added: String,
    pub due_date: Option<String>,
    pub required_action: String,
    pub known_ransomware_campaign_use: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VulnerabilityEvidenceRecord {
    pub evidence_id: u64,
    pub evidence_type: String,
    pub source: String,
    pub observed_at: String,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProductIdentityCandidate {
    pub cpe: String,
    pub canonical_vendor: String,
    pub canonical_product: String,
    pub resolution_method: String,
    pub confidence: String,
    pub provenance: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProductIdentityDetail {
    pub status: String,
    pub canonical_vendor: Option<String>,
    pub canonical_product: Option<String>,
    pub normalized_version: Option<String>,
    pub cpe_candidate: Option<String>,
    pub resolution_method: Option<String>,
    pub confidence: Option<String>,
    pub unresolved_reason: Option<String>,
    pub resolver_version: u32,
    pub provenance: Vec<String>,
    pub candidates: Vec<ProductIdentityCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareVulnerabilityDetail {
    pub summary: SoftwareVulnerabilitySummary,
    pub product_identity: Option<ProductIdentityDetail>,
    pub matches: Vec<VulnerabilityMatchRecord>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeInput {
    pub theme: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageInput {
    pub language: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveTelemetrySnapshot {
    pub collected_at: String,
    pub processes: Vec<ProcessRecord>,
    pub connections: Vec<ConnectionRecord>,
    pub services: Vec<ServiceRecord>,
    pub events: Vec<TelemetryEvent>,
    pub collectors: Vec<CollectorHealth>,
    pub issues: Vec<CollectionIssue>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessRecord {
    pub key: String,
    pub name: String,
    pub pid: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_line: Option<String>,
    pub cpu_percent: Option<f32>,
    pub core_equivalent_cpu_percent: Option<f32>,
    pub memory_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    pub signature_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable_file_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable_modified_at: Option<String>,
    pub access_status: String,
    pub first_seen: String,
    pub last_seen: String,
    pub observation_count: u64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionRecord {
    pub key: String,
    pub protocol: String,
    pub ip_version: String,
    pub local_address: String,
    pub local_port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable_path: Option<String>,
    pub association_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_last_seen: Option<String>,
    pub first_seen: String,
    pub last_seen: String,
    pub observation_count: u64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRecord {
    pub key: String,
    pub service_name: String,
    pub display_name: String,
    pub status: String,
    pub startup_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    pub first_seen: String,
    pub last_seen: String,
    pub observation_count: u64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryEvent {
    pub event_id: String,
    pub event_type: String,
    pub entity_type: String,
    pub entity_key: String,
    pub timestamp: String,
    pub collector: String,
    pub factual_payload: serde_json::Value,
    pub schema_version: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CollectorStatus {
    Healthy,
    Degraded,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectorHealth {
    pub id: String,
    pub status: CollectorStatus,
    pub detail: String,
    pub last_success: Option<String>,
    pub last_attempt: String,
    pub duration_ms: u64,
    pub observation_count: usize,
    pub restricted_count: usize,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BaselineStatus {
    NotInitialized,
    Learning,
    Ready,
    Stale,
    Error,
}

#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BaselineEntityCounts {
    pub executables: u64,
    pub process_patterns: u64,
    pub parent_child_relationships: u64,
    pub network_destinations: u64,
    pub services: u64,
    pub network_configurations: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineSummary {
    pub baseline_id: Option<String>,
    pub created_at: Option<String>,
    pub learning_started_at: Option<String>,
    pub learning_completed_at: Option<String>,
    pub version: Option<u32>,
    pub host_id: Option<String>,
    pub status: BaselineStatus,
    pub observation_count: u64,
    pub schema_version: u32,
    pub learning_period_seconds: u64,
    pub last_observed_at: Option<String>,
    pub updated_at: Option<String>,
    pub last_processing_duration_ms: u64,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub entities: BaselineEntityCounts,
}

impl Default for BaselineSummary {
    fn default() -> Self {
        Self {
            baseline_id: None,
            created_at: None,
            learning_started_at: None,
            learning_completed_at: None,
            version: None,
            host_id: None,
            status: BaselineStatus::NotInitialized,
            observation_count: 0,
            schema_version: 1,
            learning_period_seconds: 0,
            last_observed_at: None,
            updated_at: None,
            last_processing_duration_ms: 0,
            error_code: None,
            error_message: None,
            entities: BaselineEntityCounts::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityEventRecord {
    pub event_id: String,
    pub event_type: String,
    pub entity_type: String,
    pub entity_key: String,
    pub title: String,
    pub timestamp: String,
    pub first_seen: String,
    pub last_seen: String,
    pub evidence: serde_json::Value,
    pub baseline_context: serde_json::Value,
    pub source: String,
    pub baseline_id: Option<String>,
    pub rule_id: Option<String>,
    pub rule_version: Option<u32>,
    pub confidence: Option<String>,
    pub status: SecurityEventStatus,
    pub observation_count: u64,
    pub condition_active: bool,
    pub schema_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SecurityEventStatus {
    New,
    Seen,
    Acknowledged,
    Resolved,
    Ignored,
}

impl SecurityEventStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Seen => "seen",
            Self::Acknowledged => "acknowledged",
            Self::Resolved => "resolved",
            Self::Ignored => "ignored",
        }
    }

    pub fn from_persisted(value: &str) -> Result<Self, String> {
        match value {
            "new" => Ok(Self::New),
            "seen" => Ok(Self::Seen),
            "acknowledged" => Ok(Self::Acknowledged),
            "resolved" => Ok(Self::Resolved),
            "ignored" => Ok(Self::Ignored),
            _ => Err("Stored security event status is invalid".into()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineActionInput {
    pub confirmation: String,
    pub learning_period_seconds: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityEventStatusInput {
    pub event_id: String,
    pub status: SecurityEventStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum DetectionSeverity {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}

impl DetectionSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Informational => "informational",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    pub fn from_persisted(value: &str) -> Result<Self, String> {
        match value {
            "informational" => Ok(Self::Informational),
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            _ => Err("Stored detection severity is invalid".into()),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DetectionConfidence {
    Low,
    Medium,
    High,
}

impl DetectionConfidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    pub fn from_persisted(value: &str) -> Result<Self, String> {
        match value {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            _ => Err("Stored detection confidence is invalid".into()),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DetectionStatus {
    New,
    Investigating,
    Acknowledged,
    Resolved,
    Ignored,
}

impl DetectionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Investigating => "investigating",
            Self::Acknowledged => "acknowledged",
            Self::Resolved => "resolved",
            Self::Ignored => "ignored",
        }
    }

    pub fn from_persisted(value: &str) -> Result<Self, String> {
        match value {
            "new" => Ok(Self::New),
            "investigating" => Ok(Self::Investigating),
            "acknowledged" => Ok(Self::Acknowledged),
            "resolved" => Ok(Self::Resolved),
            "ignored" => Ok(Self::Ignored),
            _ => Err("Stored detection status is invalid".into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DetectionExplanation {
    pub what_happened: String,
    pub why_flagged: String,
    pub severity_reason: String,
    pub confidence_reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionRecord {
    pub detection_id: String,
    pub rule_id: String,
    pub rule_version: u32,
    pub entity_type: String,
    pub entity_key: String,
    pub title: String,
    pub summary: String,
    pub severity: DetectionSeverity,
    pub confidence: DetectionConfidence,
    pub status: DetectionStatus,
    pub first_detected_at: String,
    pub last_detected_at: String,
    pub occurrence_count: u64,
    pub baseline_id: Option<String>,
    pub explanation: DetectionExplanation,
    pub remediation_guidance: Vec<String>,
    pub condition_active: bool,
    pub schema_version: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionStatusInput {
    pub detection_id: String,
    pub status: DetectionStatus,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleEnabledInput {
    pub rule_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionEvidenceRecord {
    pub evidence_id: String,
    pub event_id: String,
    pub evidence_type: String,
    pub label: String,
    pub value: serde_json::Value,
    pub observed_at: String,
    pub source: String,
    pub event_type: String,
    pub entity_type: String,
    pub entity_key: String,
    pub event_schema_version: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScoreState {
    Available,
    Limited,
    Unavailable,
}

impl ScoreState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Limited => "limited",
            Self::Unavailable => "unavailable",
        }
    }

    pub fn from_persisted(value: &str) -> Result<Self, String> {
        match value {
            "available" => Ok(Self::Available),
            "limited" => Ok(Self::Limited),
            "unavailable" => Ok(Self::Unavailable),
            _ => Err("Stored score state is invalid".into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScoreCoverage {
    pub component: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScoreBreakdown {
    pub correlation_key: String,
    pub title: String,
    pub severity: DetectionSeverity,
    pub confidence: DetectionConfidence,
    pub penalty: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VulnerabilityCoverage {
    pub status: String,
    #[serde(default = "vulnerability_coverage_basis")]
    pub basis: String,
    pub total_software: u64,
    pub eligible_software: u64,
    pub resolved_eligible: u64,
    pub ambiguous_eligible: u64,
    pub unresolved_eligible: u64,
    pub not_mappable: u64,
    pub pending_evaluation: u64,
}

fn vulnerability_coverage_basis() -> String {
    "software_record_proxy".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct VulnerabilityCvssBands {
    pub unrated: u64,
    pub low: u64,
    pub medium: u64,
    pub high: u64,
    pub critical: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProductVulnerabilityRisk {
    pub product_risk_key: String,
    pub canonical_vendor: String,
    pub canonical_product: String,
    pub display_names: Vec<String>,
    pub software_ids: Vec<String>,
    pub installed_versions: Vec<String>,
    pub confirmed_cve_ids: Vec<String>,
    #[serde(default)]
    pub possible_cve_ids: Vec<String>,
    pub confirmed_count: u64,
    pub possible_count: u64,
    pub cvss_bands: VulnerabilityCvssBands,
    pub highest_cvss: Option<f64>,
    pub confirmed_kev_count: u64,
    pub severity_anchor: f64,
    pub marginal_breadth: f64,
    pub kev_boost: f64,
    pub uncapped_impact: f64,
    pub impact: u32,
    pub matching_engine_versions: Vec<u32>,
    pub identity_resolver_versions: Vec<u32>,
    pub nvd_source_versions: Vec<String>,
    pub kev_source_versions: Vec<String>,
    pub evaluated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityScore {
    pub state: ScoreState,
    pub score: Option<u32>,
    pub label: Option<String>,
    pub generated_at: Option<String>,
    pub formula_version: u32,
    pub active_detection_count: u64,
    pub highest_severity: Option<DetectionSeverity>,
    pub coverage: Vec<ScoreCoverage>,
    pub breakdown: Vec<ScoreBreakdown>,
    pub detection_penalty: u32,
    pub vulnerability_penalty: u32,
    pub vulnerability_coverage: VulnerabilityCoverage,
    pub product_vulnerability_risks: Vec<ProductVulnerabilityRisk>,
    pub reason: Option<String>,
}
