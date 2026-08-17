ALTER TABLE software_identity_evaluations
ADD COLUMN identity_resolver_version INTEGER NOT NULL DEFAULT 1
    CHECK (identity_resolver_version > 0);

ALTER TABLE software_identity_evaluations
ADD COLUMN resolver_status TEXT NOT NULL DEFAULT 'unresolved'
    CHECK (resolver_status IN ('resolved', 'ambiguous', 'unresolved'));

ALTER TABLE software_identity_evaluations
ADD COLUMN canonical_vendor TEXT;

ALTER TABLE software_identity_evaluations
ADD COLUMN canonical_product TEXT;

ALTER TABLE software_identity_evaluations
ADD COLUMN resolution_method TEXT
    CHECK (resolution_method IS NULL OR resolution_method IN (
        'exact_identity', 'alias_registry', 'product_extractor', 'canonical_cpe_lookup'
    ));

ALTER TABLE software_identity_evaluations
ADD COLUMN resolver_confidence TEXT
    CHECK (resolver_confidence IS NULL OR resolver_confidence IN ('high', 'medium', 'low'));

ALTER TABLE software_identity_evaluations
ADD COLUMN unresolved_reason TEXT
    CHECK (unresolved_reason IS NULL OR unresolved_reason IN (
        'no_cpe_candidate', 'vendor_mismatch', 'ambiguous_product',
        'version_unparseable', 'missing_version', 'component_not_mappable',
        'multiple_candidates', 'unsupported_version_scheme'
    ));

ALTER TABLE software_identity_evaluations
ADD COLUMN provenance_json TEXT NOT NULL DEFAULT '[]'
    CHECK (json_valid(provenance_json));

ALTER TABLE software_cpe_candidates
ADD COLUMN candidate_origin TEXT NOT NULL DEFAULT 'canonical_cpe_lookup'
    CHECK (candidate_origin IN (
        'exact_identity', 'alias_registry', 'product_extractor', 'canonical_cpe_lookup'
    ));

ALTER TABLE software_cpe_candidates
ADD COLUMN canonical_cpe TEXT;

CREATE INDEX idx_identity_evaluations_resolver_status
    ON software_identity_evaluations(resolver_status, software_id, evaluation_id);
