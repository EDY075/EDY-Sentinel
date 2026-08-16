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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeInput {
    pub theme: String,
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
    pub reason: Option<String>,
}
