const VERSION: &str = env!("CARGO_PKG_VERSION");

use cargo_metadata::MetadataCommand;
use clap::{App, AppSettings, Arg, ArgMatches};
use semver::{Identifier, SemVerError, Version};
use std::path::PathBuf;
use std::str::FromStr;

pub fn get_config() -> Config {
    let matches = build_cli_parser().get_matches();
    Config::from_matches(matches)
}

fn build_cli_parser<'a, 'b>() -> App<'a, 'b> {
    App::new("cargo-bump")
        .version(VERSION)
        .author("Wraithan McCarroll <xwraithanx@gmail.com>")
        .usage(
            "cargo bump <VERSION | major | minor | patch> [FLAGS]

    Version parts: ${PREFIX}${MAJOR}.${MINOR}.${PATCH}-${PRE-RELEASE}+${BUILD}
    Example: v3.1.4-alpha+159",
        )
        .about("Increments the version number in Cargo.toml as specified.")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version_short("v")
        .arg(
            // This is because when we're called from cargo,
            // our first arg is the command we were calld as.
            Arg::with_name("bump")
                .possible_value("bump")
                .index(1)
                .required(true)
                .hidden(true),
        )
        .arg(
            Arg::with_name("manifest-path")
                .long("manifest-path")
                .value_name("PATH")
                .takes_value(true)
                .help("Path to Cargo.toml"),
        )
        .arg(Arg::with_name("VERSION").index(2).help(
            "Must be 'major', 'minor', 'patch' or a semantic version string: https://semver.org",
        ))
        .arg(
            Arg::with_name("pre-release")
                .short("p")
                .long("pre-release")
                .value_name("PRE-RELEASE")
                .takes_value(true)
                .help("Add pre-release part to version, e.g. 'beta'"),
        )
        .arg(
            Arg::with_name("build-metadata")
                .short("b")
                .long("build")
                .value_name("BUILD")
                .takes_value(true)
                .help("Add build part to version, e.g. 'dirty'"),
        )
        .arg(
            Arg::with_name("git-tag")
                .short("g")
                .long("git-tag")
                .help("Commit the updated version and create a git tag"),
        )
        .arg(
            Arg::with_name("run-build")
                .short("r")
                .long("run-build")
                .help("Require `cargo build` to succeed (and update Cargo.lock) before running git actions"),
        )
        .arg(
            Arg::with_name("tag-prefix")
                .short("t")
                .long("tag-prefix")
                .value_name("PREFIX")
                .takes_value(true)
                .help("Prefix to the git-tag, e.g. 'v' (implies --git-tag)"),
        )
        .arg(
            Arg::with_name("ignore-lockfile")
                .long("ignore-lockfile")
                .help("Don't update Cargo.lock")
        )
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub version_modifier: VersionModifier,
    pub manifest: PathBuf,
    pub git_tag: bool,
    pub run_build: bool,
    pub prefix: String,
    pub ignore_lockfile: bool,
}

impl Default for Config {
    fn default() -> Self {
        let mut metadata_cmd = MetadataCommand::new();
        let metadata = metadata_cmd.exec().expect("get cargo metadata");
        let manifest = metadata[metadata
            .workspace_members
            .first()
            .expect("get workspace members")]
        .manifest_path
        .to_owned();
        let version_modifier = VersionModifier {
            mod_type: ModifierType::Patch,
            build_metadata: None,
            pre_release: None,
        };

        Config {
            version_modifier,
            manifest,
            git_tag: false,
            run_build: false,
            prefix: "".into(),
            ignore_lockfile: false,
        }
    }
}

impl Config {
    fn from_matches(matches: ArgMatches) -> Config {
        let mod_type = ModifierType::from_str(matches.value_of("VERSION").unwrap_or("patch"))
            .expect("Invalid semver version, expected version or major, minor, patch");
        let build_metadata = matches.value_of("build-metadata").map(parse_identifiers);
        let pre_release = matches.value_of("pre-release").map(parse_identifiers);
        let run_build = matches.is_present("run-build");
        let mut git_tag = matches.is_present("git-tag");
        let prefix = match matches.value_of("tag-prefix") {
            Some(prefix) => {
                git_tag = true;
                prefix.to_string()
            }
            None => "".to_string(),
        };
        let ignore_lockfile = matches.is_present("ignore-lockfile");
        let mut metadata_cmd = MetadataCommand::new();
        if let Some(path) = matches.value_of("manifest-path") {
            metadata_cmd.manifest_path(path);
        }
        let metadata = metadata_cmd.exec().expect("get cargo metadata");
        if metadata.workspace_members.len() == 1 {
            Config {
                version_modifier: VersionModifier {
                    mod_type,
                    build_metadata,
                    pre_release,
                },
                manifest: metadata[&metadata.workspace_members[0]]
                    .manifest_path
                    .clone(),
                git_tag,
                run_build,
                prefix,
                ignore_lockfile,
            }
        } else {
            panic!("Workspaces are not supported yet.");
        }
    }
}

fn parse_identifiers(value: &str) -> Vec<Identifier> {
    value
        .split('.')
        .map(|identifier| {
            if let Ok(i) = identifier.parse() {
                Identifier::Numeric(i)
            } else {
                Identifier::AlphaNumeric(identifier.to_string())
            }
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModifierType {
    Replace(Version),
    Major,
    Minor,
    Patch,
}

impl FromStr for ModifierType {
    type Err = SemVerError;
    fn from_str(input: &str) -> Result<ModifierType, Self::Err> {
        Ok(match input {
            "major" => ModifierType::Major,
            "minor" => ModifierType::Minor,
            "patch" => ModifierType::Patch,
            _ => ModifierType::Replace(Version::parse(input)?),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VersionModifier {
    pub mod_type: ModifierType,
    pub build_metadata: Option<Vec<Identifier>>,
    pub pre_release: Option<Vec<Identifier>>,
}

impl VersionModifier {
    #[allow(unused)]
    pub fn new(
        mod_type: ModifierType,
        pre_release: Option<&str>,
        build_metadata: Option<&str>,
    ) -> Self {
        Self {
            mod_type,
            build_metadata: build_metadata.map(parse_identifiers),
            pre_release: pre_release.map(parse_identifiers),
        }
    }

    #[allow(unused)]
    pub fn from_mod_type(mod_type: ModifierType) -> Self {
        Self {
            mod_type,
            build_metadata: None,
            pre_release: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn test_config(input: Vec<&str>, version_mod: VersionModifier) {
        let parser = build_cli_parser();
        let root = env::current_dir().unwrap();
        let mut manifest = root.clone();
        manifest.push("Cargo.toml");
        let matches = parser.get_matches_from_safe(input).unwrap();
        let config = Config::from_matches(matches);
        assert_eq!(config.version_modifier, version_mod);
        assert_eq!(config.manifest, manifest);
    }

    #[test]
    fn bump_arg_only() {
        let input = vec!["cargo-bump", "bump"];
        test_config(input, VersionModifier::from_mod_type(ModifierType::Patch))
    }

    #[test]
    fn version_arg_minor() {
        let input = vec!["cargo-bump", "bump", "minor"];
        test_config(input, VersionModifier::from_mod_type(ModifierType::Minor))
    }

    #[test]
    fn version_arg_string_good() {
        let input = vec!["cargo-bump", "bump", "1.2.3"];
        test_config(
            input,
            VersionModifier::from_mod_type(ModifierType::Replace(Version::parse("1.2.3").unwrap())),
        )
    }

    #[test]
    fn version_bump_and_build() {
        let input = vec!["cargo-bump", "bump", "major", "--build", "1999"];
        let version_mod = VersionModifier {
            mod_type: ModifierType::Major,
            build_metadata: Some(vec![Identifier::Numeric(1999)]),
            pre_release: None,
        };
        test_config(input, version_mod);
    }

    #[test]
    fn version_bump_and_pre() {
        let input = vec!["cargo-bump", "bump", "2.0.0", "--pre-release", "beta"];
        let version_mod = VersionModifier {
            mod_type: ModifierType::Replace(Version::parse("2.0.0").unwrap()),
            build_metadata: None,
            pre_release: Some(vec![Identifier::AlphaNumeric(String::from("beta"))]),
        };
        test_config(input, version_mod);
    }
}
