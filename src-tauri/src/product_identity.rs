pub(crate) const IDENTITY_RESOLVER_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RawProductIdentity<'a> {
    pub display_name: &'a str,
    pub display_version: Option<&'a str>,
    pub publisher: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalIdentityRequest {
    pub vendor: &'static str,
    pub product: &'static str,
    pub normalized_version: String,
    pub cpe_update: Option<String>,
    pub resolution_method: &'static str,
    pub legacy_resolution: &'static str,
    pub confidence: &'static str,
    pub provenance: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedIdentity {
    pub normalized_name: String,
    pub normalized_publisher: String,
    pub normalized_version: Option<String>,
    pub canonical: Option<CanonicalIdentityRequest>,
    pub allow_exact_lookup: bool,
    pub unresolved_reason: Option<&'static str>,
    pub provenance: Vec<String>,
}

struct Alias {
    id: &'static str,
    name: &'static str,
    publishers: &'static [&'static str],
    vendor: &'static str,
    product: &'static str,
    minimum_major: Option<u128>,
    maximum_major: Option<u128>,
}

const ALIAS_REGISTRY_VERSION: u32 = 1;
const EXTRACTOR_REGISTRY_VERSION: u32 = 1;

const ALIASES: &[Alias] = &[
    Alias {
        id: "google-chrome-windows",
        name: "google chrome",
        publishers: &["google llc"],
        vendor: "google",
        product: "chrome",
        minimum_major: None,
        maximum_major: None,
    },
    Alias {
        id: "microsoft-edge-legacy",
        name: "microsoft edge",
        publishers: &["microsoft corporation"],
        vendor: "microsoft",
        product: "edge",
        minimum_major: None,
        maximum_major: Some(18),
    },
    Alias {
        id: "microsoft-edge-chromium",
        name: "microsoft edge",
        publishers: &["microsoft corporation"],
        vendor: "microsoft",
        product: "edge_chromium",
        minimum_major: Some(79),
        maximum_major: None,
    },
    Alias {
        id: "git-for-windows",
        name: "git",
        publishers: &["the git development community"],
        vendor: "git-scm",
        product: "git",
        minimum_major: None,
        maximum_major: None,
    },
    Alias {
        id: "nodejs-windows-msi",
        name: "node js",
        publishers: &["node js foundation", "openjs foundation"],
        vendor: "nodejs",
        product: "node.js",
        minimum_major: None,
        maximum_major: None,
    },
    Alias {
        id: "discord-stable-windows",
        name: "discord",
        publishers: &["discord inc"],
        vendor: "discord",
        product: "discord",
        minimum_major: None,
        maximum_major: None,
    },
    Alias {
        id: "qbittorrent-windows",
        name: "qbittorrent",
        publishers: &["the qbittorrent project"],
        vendor: "qbittorrent",
        product: "qbittorrent",
        minimum_major: None,
        maximum_major: None,
    },
];

pub(crate) fn prepare(raw: RawProductIdentity<'_>) -> PreparedIdentity {
    let normalized_name = normalize(raw.display_name);
    let normalized_publisher = raw.publisher.map(normalize).unwrap_or_default();
    let base_provenance = vec![format!("normalization:v{}", IDENTITY_RESOLVER_VERSION)];

    if let Some(reason) = deliberately_unmapped_reason(&normalized_name) {
        return PreparedIdentity {
            normalized_name,
            normalized_publisher,
            normalized_version: safe_version(raw.display_version),
            canonical: None,
            allow_exact_lookup: false,
            unresolved_reason: Some(reason),
            provenance: base_provenance,
        };
    }

    if let Some(alias) = ALIASES.iter().find(|alias| {
        normalized_name == alias.name
            && alias.publishers.contains(&normalized_publisher.as_str())
            && alias_version_matches(alias, raw.display_version)
    }) {
        return canonical_preparation(
            normalized_name,
            normalized_publisher,
            raw.display_version,
            alias.vendor,
            alias.product,
            None,
            "alias_registry",
            "controlled_alias",
            format!("alias_registry:v{}:{}", ALIAS_REGISTRY_VERSION, alias.id),
        );
    }

    if let Some(prepared) = extract_product(raw, &normalized_name, &normalized_publisher) {
        return prepared;
    }

    let normalized_version = safe_version(raw.display_version);
    let unresolved_reason = if raw.display_version.is_none() {
        Some("missing_version")
    } else if normalized_version.is_none() {
        Some("version_unparseable")
    } else {
        None
    };
    PreparedIdentity {
        normalized_name,
        normalized_publisher,
        normalized_version,
        canonical: None,
        allow_exact_lookup: unresolved_reason.is_none(),
        unresolved_reason,
        provenance: base_provenance,
    }
}

#[allow(clippy::too_many_arguments)]
fn canonical_preparation(
    normalized_name: String,
    normalized_publisher: String,
    display_version: Option<&str>,
    vendor: &'static str,
    product: &'static str,
    cpe_update: Option<String>,
    resolution_method: &'static str,
    legacy_resolution: &'static str,
    provenance: String,
) -> PreparedIdentity {
    let mut pipeline = vec![format!("normalization:v{}", IDENTITY_RESOLVER_VERSION)];
    pipeline.push(provenance);
    let normalized_version = safe_version(display_version);
    let unresolved_reason = if display_version.is_none() {
        Some("missing_version")
    } else if normalized_version.is_none() {
        Some("version_unparseable")
    } else {
        None
    };
    let canonical = normalized_version
        .as_ref()
        .map(|version| CanonicalIdentityRequest {
            vendor,
            product,
            normalized_version: version.clone(),
            cpe_update,
            resolution_method,
            legacy_resolution,
            confidence: "high",
            provenance: pipeline.clone(),
        });
    PreparedIdentity {
        normalized_name,
        normalized_publisher,
        normalized_version,
        canonical,
        allow_exact_lookup: false,
        unresolved_reason,
        provenance: pipeline,
    }
}

fn extract_product(
    raw: RawProductIdentity<'_>,
    normalized_name: &str,
    normalized_publisher: &str,
) -> Option<PreparedIdentity> {
    if normalized_name.starts_with("python ")
        && normalized_publisher == "python software foundation"
    {
        if normalized_name == "python launcher" {
            return None;
        }
        let version = extract_python_version(raw);
        return Some(extractor_preparation(
            raw,
            normalized_name,
            normalized_publisher,
            version,
            "python",
            "python",
            None,
            "python-registry-display-name",
        ));
    }

    if normalized_name.starts_with("java ") && normalized_publisher == "oracle corporation" {
        if raw.display_version.is_none() {
            return Some(unresolved_extractor(
                normalized_name,
                normalized_publisher,
                "missing_version",
                "oracle-java-8-update",
            ));
        }
        let Some((version, edition)) = extract_oracle_java_8(raw) else {
            return Some(unresolved_extractor(
                normalized_name,
                normalized_publisher,
                "unsupported_version_scheme",
                "oracle-java-8-update",
            ));
        };
        return Some(extractor_preparation(
            raw,
            normalized_name,
            normalized_publisher,
            Some(version),
            "oracle",
            "jre",
            Some(edition),
            "oracle-java-8-update",
        ));
    }

    if normalized_name.starts_with("winrar ") && normalized_publisher == "win rar gmbh" {
        return Some(extractor_preparation(
            raw,
            normalized_name,
            normalized_publisher,
            versioned_name(raw, "WinRAR "),
            "rarlab",
            "winrar",
            None,
            "winrar-display-name",
        ));
    }

    if normalized_name.starts_with("opera stable ") && normalized_publisher == "opera software" {
        return Some(extractor_preparation(
            raw,
            normalized_name,
            normalized_publisher,
            exact_name_suffix_version(raw, "Opera Stable "),
            "opera",
            "opera_browser",
            None,
            "opera-stable-display-name",
        ));
    }

    if normalized_name.starts_with("oracle virtualbox ")
        && normalized_publisher == "oracle and or its affiliates"
    {
        return Some(extractor_preparation(
            raw,
            normalized_name,
            normalized_publisher,
            exact_name_suffix_version(raw, "Oracle VirtualBox "),
            "oracle",
            "vm_virtualbox",
            None,
            "oracle-virtualbox-display-name",
        ));
    }

    if normalized_name.starts_with("php ") && normalized_publisher == "php group" {
        let version = strip_prefix_ascii_case(raw.display_name, "PHP ").and_then(|family| {
            let installed = safe_version(raw.display_version)?;
            (installed == family || installed.starts_with(&format!("{family}.")))
                .then_some(installed)
        });
        return Some(extractor_preparation(
            raw,
            normalized_name,
            normalized_publisher,
            version,
            "php",
            "php",
            None,
            "php-windows-family",
        ));
    }

    None
}

#[allow(clippy::too_many_arguments)]
fn extractor_preparation(
    raw: RawProductIdentity<'_>,
    normalized_name: &str,
    normalized_publisher: &str,
    version: Option<String>,
    vendor: &'static str,
    product: &'static str,
    cpe_update: Option<String>,
    extractor_id: &'static str,
) -> PreparedIdentity {
    let provenance = vec![
        format!("normalization:v{}", IDENTITY_RESOLVER_VERSION),
        format!(
            "product_extractor:v{}:{}",
            EXTRACTOR_REGISTRY_VERSION, extractor_id
        ),
    ];
    let unresolved_reason = if raw.display_version.is_none() {
        Some("missing_version")
    } else if version.is_none() {
        Some("version_unparseable")
    } else {
        None
    };
    let canonical = if unresolved_reason.is_none() {
        version
            .as_ref()
            .map(|normalized_version| CanonicalIdentityRequest {
                vendor,
                product,
                normalized_version: normalized_version.clone(),
                cpe_update,
                resolution_method: "product_extractor",
                legacy_resolution: "controlled_alias",
                confidence: "high",
                provenance: provenance.clone(),
            })
    } else {
        None
    };
    PreparedIdentity {
        normalized_name: normalized_name.into(),
        normalized_publisher: normalized_publisher.into(),
        normalized_version: version,
        canonical,
        allow_exact_lookup: false,
        unresolved_reason,
        provenance,
    }
}

fn unresolved_extractor(
    normalized_name: &str,
    normalized_publisher: &str,
    reason: &'static str,
    extractor_id: &'static str,
) -> PreparedIdentity {
    PreparedIdentity {
        normalized_name: normalized_name.into(),
        normalized_publisher: normalized_publisher.into(),
        normalized_version: None,
        canonical: None,
        allow_exact_lookup: false,
        unresolved_reason: Some(reason),
        provenance: vec![
            format!("normalization:v{}", IDENTITY_RESOLVER_VERSION),
            format!(
                "product_extractor:v{}:{}",
                EXTRACTOR_REGISTRY_VERSION, extractor_id
            ),
        ],
    }
}

fn extract_python_version(raw: RawProductIdentity<'_>) -> Option<String> {
    let suffix = strip_prefix_ascii_case(raw.display_name, "Python ")?;
    let (version, architecture) = suffix.rsplit_once(" (")?;
    if !["64-bit)", "32-bit)", "arm64)"]
        .iter()
        .any(|expected| architecture.eq_ignore_ascii_case(expected))
    {
        return None;
    }
    let parts = version.split('.').collect::<Vec<_>>();
    let extracted = (matches!(parts.len(), 2 | 3)
        && parts.iter().all(|part| {
            !part.is_empty() && part.chars().all(|character| character.is_ascii_digit())
        }))
    .then(|| version.to_ascii_lowercase())?;
    let registry_version = safe_version(raw.display_version)?;
    let family = parts.iter().take(2).copied().collect::<Vec<_>>().join(".");
    (registry_version == family || registry_version.starts_with(&format!("{family}.")))
        .then_some(extracted)
}

fn extract_oracle_java_8(raw: RawProductIdentity<'_>) -> Option<(String, String)> {
    let normalized = normalize(raw.display_name);
    let words = normalized.split_whitespace().collect::<Vec<_>>();
    let (major, update) = match words.as_slice() {
        ["java", major, "update", update]
        | ["java", major, "update", update, "64", "bit"]
        | ["java", major, "update", update, "32", "bit"] => (*major, *update),
        _ => return None,
    };
    if major != "8" || !update.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    let registry_version = safe_version(raw.display_version)?;
    let expected_prefix = format!("8.0.{update}0.");
    if !registry_version.starts_with(&expected_prefix) {
        return None;
    }
    Some(("1.8.0".into(), format!("update{update}")))
}

fn versioned_name(raw: RawProductIdentity<'_>, prefix: &str) -> Option<String> {
    let suffix = strip_prefix_ascii_case(raw.display_name, prefix)?;
    let (embedded, architecture) = suffix.split_once(' ')?;
    if !["(64-bit)", "(32-bit)"]
        .iter()
        .any(|expected| architecture.eq_ignore_ascii_case(expected))
    {
        return None;
    }
    let embedded = safe_version(Some(embedded))?;
    let installed = safe_version(raw.display_version)?;
    (installed == embedded || installed.starts_with(&format!("{embedded}."))).then_some(installed)
}

fn exact_name_suffix_version(raw: RawProductIdentity<'_>, prefix: &str) -> Option<String> {
    let embedded = strip_prefix_ascii_case(raw.display_name, prefix)?;
    let installed = safe_version(raw.display_version)?;
    embedded
        .eq_ignore_ascii_case(&installed)
        .then_some(installed)
}

fn strip_prefix_ascii_case<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    let candidate = value.get(..prefix.len())?;
    candidate
        .eq_ignore_ascii_case(prefix)
        .then(|| &value[prefix.len()..])
}

fn alias_version_matches(alias: &Alias, version: Option<&str>) -> bool {
    if alias.minimum_major.is_none() && alias.maximum_major.is_none() {
        return safe_version(version).is_some();
    }
    let Some(major) = version
        .and_then(|value| value.split(['.', '-', '_', '+']).next())
        .and_then(|value| value.parse::<u128>().ok())
    else {
        return false;
    };
    alias.minimum_major.map_or(true, |minimum| major >= minimum)
        && alias.maximum_major.map_or(true, |maximum| major <= maximum)
}

fn deliberately_unmapped_reason(name: &str) -> Option<&'static str> {
    (name == "python launcher"
        || name == "microsoft visual studio installer"
        || name == "ferramentas de build do visual studio 2022"
        || name.starts_with("microsoft visual c ")
        || name.ends_with(" launcher")
        || name.ends_with(" helper"))
    .then_some("component_not_mappable")
}

pub(crate) fn safe_version(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty()
        || value.len() > 128
        || value.eq_ignore_ascii_case("unresolved")
        || value.chars().any(|character| {
            !character.is_ascii_alphanumeric() && !matches!(character, '.' | '-' | '_' | '+')
        })
    {
        return None;
    }
    Some(value.to_ascii_lowercase())
}

pub(crate) fn normalize(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw<'a>(
        name: &'a str,
        version: Option<&'a str>,
        publisher: Option<&'a str>,
    ) -> RawProductIdentity<'a> {
        RawProductIdentity {
            display_name: name,
            display_version: version,
            publisher,
        }
    }

    #[test]
    fn node_alias_is_normalized_and_rejects_wrong_vendor_or_similar_names() {
        let valid = prepare(raw("Node.js", Some("24.17.0"), Some("Node.js Foundation")));
        assert_eq!(
            valid
                .canonical
                .as_ref()
                .map(|item| (item.vendor, item.product)),
            Some(("nodejs", "node.js"))
        );
        assert_eq!(
            valid
                .canonical
                .as_ref()
                .map(|item| item.normalized_version.as_str()),
            Some("24.17.0")
        );
        assert!(
            prepare(raw("Node.js", Some("24.17.0"), Some("Different Vendor")))
                .canonical
                .is_none()
        );
        assert!(prepare(raw(
            "Node.js Helper",
            Some("24.17.0"),
            Some("Node.js Foundation")
        ))
        .canonical
        .is_none());
        assert!(
            prepare(raw("Nodejs", Some("24.17.0"), Some("Node.js Foundation")))
                .canonical
                .is_none()
        );
    }

    #[test]
    fn python_extractor_uses_display_name_version_not_registry_package_build() {
        let valid = prepare(raw(
            "Python 3.12.10 (64-bit)",
            Some("3.12.10150.0"),
            Some("Python Software Foundation"),
        ));
        let canonical = valid.canonical.expect("Python identity");
        assert_eq!(
            (
                canonical.vendor,
                canonical.product,
                canonical.normalized_version.as_str()
            ),
            ("python", "python", "3.12.10")
        );
        assert_eq!(canonical.resolution_method, "product_extractor");
        assert!(prepare(raw(
            "Python 3.12.10 (64-bit)",
            Some("3.12.10150.0"),
            Some("Other Foundation")
        ))
        .canonical
        .is_none());
        assert_eq!(
            prepare(raw(
                "Python Launcher",
                Some("3.12.10150.0"),
                Some("Python Software Foundation")
            ))
            .unresolved_reason,
            Some("component_not_mappable")
        );
        assert_eq!(
            prepare(raw(
                "Python 3.12 (64-bit)",
                None,
                Some("Python Software Foundation")
            ))
            .unresolved_reason,
            Some("missing_version")
        );
        assert_eq!(
            prepare(raw(
                "Python release (64-bit)",
                Some("invalid version"),
                Some("Python Software Foundation")
            ))
            .unresolved_reason,
            Some("version_unparseable")
        );
    }

    #[test]
    fn oracle_java_extractor_preserves_the_update_dimension_and_distribution() {
        let valid = prepare(raw(
            "Java 8 Update 401 (64-bit)",
            Some("8.0.4010.10"),
            Some("Oracle Corporation"),
        ));
        let canonical = valid.canonical.expect("Oracle JRE identity");
        assert_eq!((canonical.vendor, canonical.product), ("oracle", "jre"));
        assert_eq!(canonical.normalized_version, "1.8.0");
        assert_eq!(canonical.cpe_update.as_deref(), Some("update401"));
        assert!(prepare(raw(
            "Java 8 Update 401 (64-bit)",
            Some("8.0.4010.10"),
            Some("Eclipse Adoptium")
        ))
        .canonical
        .is_none());
        assert_eq!(
            prepare(raw(
                "Java 8 Update 401 (64-bit)",
                Some("8.0.4020.10"),
                Some("Oracle Corporation")
            ))
            .unresolved_reason,
            Some("unsupported_version_scheme")
        );
        assert!(prepare(raw(
            "Java Development Kit 8 Update 401",
            Some("8.0.4010.10"),
            Some("Oracle Corporation")
        ))
        .canonical
        .is_none());
    }

    #[test]
    fn installed_aliases_cover_exact_products_without_broad_substrings() {
        for (name, version, publisher, vendor, product) in [
            (
                "Google Chrome",
                "151.0.7922.138",
                "Google LLC",
                "google",
                "chrome",
            ),
            (
                "Microsoft Edge",
                "151.0.4129.86",
                "Microsoft Corporation",
                "microsoft",
                "edge_chromium",
            ),
            (
                "Git",
                "2.55.0.3",
                "The Git Development Community",
                "git-scm",
                "git",
            ),
            ("Discord", "1.0.9254", "Discord Inc.", "discord", "discord"),
            (
                "qBittorrent",
                "5.2.3",
                "The qBittorrent project",
                "qbittorrent",
                "qbittorrent",
            ),
        ] {
            let canonical = prepare(raw(name, Some(version), Some(publisher)))
                .canonical
                .expect(name);
            assert_eq!((canonical.vendor, canonical.product), (vendor, product));
            assert!(prepare(raw(name, Some(version), Some("Different Vendor")))
                .canonical
                .is_none());
            assert!(prepare(raw(
                &format!("{name} Helper"),
                Some(version),
                Some(publisher)
            ))
            .canonical
            .is_none());
            assert!(prepare(raw(
                &format!("Other {name}"),
                Some(version),
                Some(publisher)
            ))
            .canonical
            .is_none());
            assert!(prepare(raw(name, None, Some(publisher)))
                .canonical
                .is_none());
            assert!(prepare(raw(name, Some("invalid version"), Some(publisher)))
                .canonical
                .is_none());
            let upper_name = name.to_ascii_uppercase();
            let upper_publisher = publisher.to_ascii_uppercase();
            let uppercase = prepare(raw(&upper_name, Some(version), Some(&upper_publisher)))
                .canonical
                .expect("ASCII casing must not change an exact identity");
            assert_eq!((uppercase.vendor, uppercase.product), (vendor, product));
        }
        assert!(
            prepare(raw("Discord PTB", Some("1.0.1214"), Some("Discord Inc.")))
                .canonical
                .is_none()
        );
        assert!(prepare(raw(
            "QBitTorrent Helper",
            Some("5.2.3"),
            Some("The qBittorrent project")
        ))
        .canonical
        .is_none());
        assert!(
            prepare(raw("Dｉscord", Some("1.0.9254"), Some("Discord Inc.")))
                .canonical
                .is_none()
        );
    }

    #[test]
    fn installed_product_extractors_require_exact_vendor_name_and_version_relationships() {
        for (name, version, publisher, vendor, product) in [
            (
                "WinRAR 7.22 (64-bit)",
                "7.22.0",
                "win.rar GmbH",
                "rarlab",
                "winrar",
            ),
            (
                "Opera Stable 134.0.5954.46",
                "134.0.5954.46",
                "Opera Software",
                "opera",
                "opera_browser",
            ),
            (
                "Oracle VirtualBox 7.2.12",
                "7.2.12",
                "Oracle and/or its affiliates",
                "oracle",
                "vm_virtualbox",
            ),
            ("PHP 8.4", "8.4.24", "PHP Group", "php", "php"),
        ] {
            let canonical = prepare(raw(name, Some(version), Some(publisher)))
                .canonical
                .expect(name);
            assert_eq!((canonical.vendor, canonical.product), (vendor, product));
            assert!(prepare(raw(name, Some(version), Some("Different Vendor")))
                .canonical
                .is_none());
            assert!(prepare(raw(
                &format!("{name} Helper"),
                Some(version),
                Some(publisher)
            ))
            .canonical
            .is_none());
            assert!(prepare(raw(
                &format!("Other {name}"),
                Some(version),
                Some(publisher)
            ))
            .canonical
            .is_none());
            assert!(prepare(raw(name, None, Some(publisher)))
                .canonical
                .is_none());
            assert!(prepare(raw(name, Some("invalid version"), Some(publisher)))
                .canonical
                .is_none());
            let upper_name = name.to_ascii_uppercase();
            let upper_publisher = publisher.to_ascii_uppercase();
            let uppercase = prepare(raw(&upper_name, Some(version), Some(&upper_publisher)))
                .canonical
                .expect("ASCII casing must not change an extracted identity");
            assert_eq!((uppercase.vendor, uppercase.product), (vendor, product));
        }
        assert!(prepare(raw(
            "Opera Stable 134.0.5954.46",
            Some("135.0.1"),
            Some("Opera Software")
        ))
        .canonical
        .is_none());
        assert!(prepare(raw(
            "Oracle VirtualBox Manager 7.2.12",
            Some("7.2.12"),
            Some("Oracle and/or its affiliates")
        ))
        .canonical
        .is_none());
        assert!(
            prepare(raw("PHP 8.4", Some("8.4.24"), Some("Different Vendor")))
                .canonical
                .is_none()
        );
    }
}
