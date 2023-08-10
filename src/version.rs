use std::str::FromStr;

use config::{ModifierType, VersionModifier};
use semver::{Prerelease, Version};

pub fn update_version(version: &mut Version, modifier: VersionModifier) {
    match modifier.mod_type {
        ModifierType::Replace(v) => {
            *version = v;
        }
        ModifierType::Major => {
            version.major += 1;
            version.minor = 0;
            version.patch = 0;
        }
        ModifierType::Minor => {
            version.minor += 1;
            version.patch = 0;
        }
        ModifierType::Patch => {
            version.patch += 1;
        }
    }

    if let Some(pre) = modifier.pre_release {
        version.pre = Prerelease::from_str(&pre).unwrap();
    }
    if let Some(build) = modifier.build_metadata {
        version.build = build;
    }
}
