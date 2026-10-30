use super::*;
use crate::config_toml::ConfigToml;
use pretty_assertions::assert_eq;

#[test]
fn computer_use_config_round_trips() {
    let config: ConfigToml = toml::from_str(
        r#"
[computer_use]
default_app_access = "deny"

[computer_use.macos.bundle_ids]
"com.apple.Safari" = "allow"

[computer_use.windows.aumids]
"Microsoft.Paint_8wekyb3d8bbwe!App" = "deny"

[[computer_use.windows.exes]]
publisher_name = "CN=Google LLC"
product_name = "Google Chrome"
binary_name = "chrome.exe"
access = "allow"
"#,
    )
    .expect("computer use config should deserialize");

    let expected = ComputerUseConfigToml {
        default_app_access: Some(AllowDenyRequirementToml::Deny),
        macos: Some(ComputerUseMacosConfigToml {
            bundle_ids: Some(BTreeMap::from([(
                "com.apple.Safari".to_string(),
                AllowDenyRequirementToml::Allow,
            )])),
        }),
        windows: Some(ComputerUseWindowsConfigToml {
            always_allowed_app_ids: None,
            aumids: Some(BTreeMap::from([(
                "Microsoft.Paint_8wekyb3d8bbwe!App".to_string(),
                AllowDenyRequirementToml::Deny,
            )])),
            exes: Some(vec![ComputerUseWindowsExeConfigToml {
                publisher_name: "CN=Google LLC".to_string(),
                product_name: "Google Chrome".to_string(),
                binary_name: Some("chrome.exe".to_string()),
                access: AllowDenyRequirementToml::Allow,
            }]),
        }),
    };
    assert_eq!(config.computer_use, Some(expected.clone()));

    let serialized = toml::to_string(&config).expect("computer use config should serialize");
    let reparsed: ConfigToml =
        toml::from_str(&serialized).expect("serialized computer use config should deserialize");
    assert_eq!(reparsed.computer_use, Some(expected));
}

#[test]
fn desktop_saved_decisions_survive_config_toml_round_trip() {
    let input = "[computer_use.windows.always_allowed_app_ids]\n\"fixture.exe\" = true\n\"declined.exe\" = false\n";
    let parsed: ConfigToml = toml::from_str(input).unwrap();
    let decisions = parsed
        .computer_use
        .as_ref()
        .unwrap()
        .windows
        .as_ref()
        .unwrap()
        .always_allowed_app_ids
        .as_ref()
        .unwrap();
    assert_eq!(decisions.get("fixture.exe"), Some(&true));
    assert_eq!(decisions.get("declined.exe"), Some(&false));
    let again: ConfigToml = toml::from_str(&toml::to_string(&parsed).unwrap()).unwrap();
    assert_eq!(again.computer_use, parsed.computer_use);
}

#[test]
fn desktop_saved_decisions_are_seen_by_strict_config_tracking() {
    let input = "[computer_use.windows.always_allowed_app_ids]\n\"fixture.exe\" = true\n";
    assert!(
        crate::strict_config::config_error_from_ignored_toml_fields::<ConfigToml>(
            std::path::Path::new("fixture.toml"),
            input,
        )
        .is_none()
    );
}

#[test]
fn malformed_decision_and_unrelated_unknown_key_are_still_rejected() {
    let malformed = "[computer_use.windows.always_allowed_app_ids]\n\"fixture.exe\" = \"allow\"\n";
    assert!(toml::from_str::<ConfigToml>(malformed).is_err());
    let unknown = "unexpected_fixture = true\n[computer_use.windows.always_allowed_app_ids]\n\"fixture.exe\" = true\n";
    assert!(
        crate::strict_config::config_error_from_ignored_toml_fields::<ConfigToml>(
            std::path::Path::new("fixture.toml"),
            unknown,
        )
        .is_some()
    );
}
