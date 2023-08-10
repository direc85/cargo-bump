use cargo_metadata::MetadataCommand;
use clap::{ArgAction, Parser};
use semver::{BuildMetadata, Error as SemVerError, Prerelease, Version};
use std::path::PathBuf;
use std::str::FromStr;

const USAGE: &str = "cargo bump <SEMVER | major | minor | patch> [FLAGS]";

#[derive(Parser, Debug)]
#[clap(author, about, version, long_about = None)]
#[clap(override_usage = USAGE)]
#[clap(help_template = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

    Version: ${PREFIX}${MAJOR}.${MINOR}.${PATCH}-${PRE-RELEASE}+${BUILD}
    Example: v3.1.4-alpha+159

{all-args}{after-help}
")]
struct Arguments {
    /// Must be 'major', 'minor', 'patch' or a semantic version string: https://semver.org
    semver: Option<String>,

    /// Path to Cargo.toml
    #[clap(value_name = "PATH", long = "manitest-path")]
    manifest_path: Option<String>,

    /// Add pre-release part to version, e.g. 'beta'
    #[clap(short = 'p', long = "pre-release", value_name = "PRE-RELEASE")]
    pre_release: Option<String>,

    /// Add build part to version, e.g. 'dirty'
    #[clap(short = 'b', long = "build", value_name = "BUILD")]
    build_metadata: Option<String>,

    /// Commit the updated version and create a git tag
    #[clap(action = ArgAction::SetTrue, short = 'g', long = "git-tag")]
    git_tag: Option<bool>,

    /// Require `cargo build` to succeed (and update Cargo.lock) before running git actions
    #[clap(action = ArgAction::SetTrue, short = 'r', long = "run-build")]
    run_build: Option<bool>,

    /// Prefix to the git-tag, e.g. 'v' (implies --git-tag)
    #[clap(short = 't', long = "tag-prefix", value_name = "PREFIX")]
    tag_prefix: Option<String>,

    /// Don't update Cargo.lock
    #[clap(action = ArgAction::SetTrue, long = "ignore-lockfile")]
    ignore_lockfile: Option<bool>,
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
        let manifest = Config::get_manifest(None);
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
    pub fn from_os_args() -> Config {
        // This is a bit ugly, but we can't use 'bump: String' in Arguments,
        // because it makes Clap ignore '--help' altogether.
        let arguments = if std::env::args_os().len() < 2
            || std::env::args_os().position(|a| a == "bump") != Some(1)
            || std::env::args_os().any(|a| a == "--help")
        {
            Arguments::parse_from(["bump", "--help"])
        } else {
            Arguments::parse_from(std::env::args_os().skip(1))
        };
        Config::parse(arguments)
    }

    fn parse(arguments: Arguments) -> Config {
        let mod_type =
            match ModifierType::from_str(&arguments.semver.unwrap_or_else(|| "patch".into())) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Invalid semver version, expected version or major, minor, patch:");
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            };
        let run_build = arguments.run_build.unwrap_or(false);
        let mut git_tag = arguments.git_tag.unwrap_or(false);

        let build_metadata = match arguments.build_metadata {
            Some(s) => BuildMetadata::from_str(&s).ok(),
            None => None,
        };
        let pre_release = match arguments.pre_release {
            Some(s) => Prerelease::from_str(&s).ok(),
            None => None,
        };

        let prefix = match arguments.tag_prefix {
            Some(prefix) => {
                git_tag = true;
                prefix
            }
            None => "".to_string(),
        };
        let ignore_lockfile = arguments.ignore_lockfile.unwrap_or(false);
        let manifest = Config::get_manifest(arguments.manifest_path);

        Config {
            version_modifier: VersionModifier {
                mod_type,
                build_metadata,
                pre_release,
            },
            manifest,
            git_tag,
            run_build,
            prefix,
            ignore_lockfile,
        }
    }

    fn get_manifest(path: Option<String>) -> PathBuf {
        let mut metadata_cmd = MetadataCommand::new();
        if let Some(path) = path {
            metadata_cmd.manifest_path(path);
        }
        let metadata = metadata_cmd.exec().expect("get cargo metadata");
        if metadata.workspace_members.len() > 1 {
            eprintln!("Workspaces are not supported yet.");
            std::process::exit(1);
        }
        let workspace = metadata
            .workspace_members
            .first()
            .expect("get workspace members");
        metadata[workspace].manifest_path.clone().into()
    }
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
    fn from_str(input: &str) -> Result<ModifierType, SemVerError> {
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
    pub build_metadata: Option<BuildMetadata>,
    pub pre_release: Option<Prerelease>,
}

impl VersionModifier {
    #[allow(unused)]
    pub fn new(
        mod_type: ModifierType,
        pre_release: Option<Prerelease>,
        build_metadata: Option<BuildMetadata>,
    ) -> Self {
        Self {
            mod_type,
            build_metadata,
            pre_release,
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

    fn test_config(input: Vec<&str>, expected_config: &Config) {
        let parser = build_cli_parser();
        let root = env::current_dir().unwrap();
        let mut manifest = root.clone();
        manifest.push("Cargo.toml");
        let matches = parser.get_matches_from_safe(input).unwrap();
        let config = Config::from_matches(matches);
        assert_eq!(&config, expected_config);
    }

    #[test]
    fn bump_arg_only() {
        let input = vec!["cargo-bump"];
        let config = Config {
            version_modifier: VersionModifier::from_mod_type(ModifierType::Patch),
            ..Default::default()
        };
        test_config(input, &config)
    }

    #[test]
    fn version_arg_minor() {
        let input = vec!["cargo-bump", "minor"];
        let config = Config {
            version_modifier: VersionModifier::from_mod_type(ModifierType::Minor),
            ..Default::default()
        };
        test_config(input, &config)
    }

    #[test]
    fn version_arg_string_good() {
        let input = vec!["cargo-bump", "1.2.3"];
        let config = Config {
            version_modifier: VersionModifier::from_mod_type(ModifierType::Replace(
                Version::parse("1.2.3").unwrap(),
            )),
            ..Default::default()
        };
        test_config(input, &config)
    }

    #[test]
    fn version_bump_and_build() {
        let input = vec!["cargo-bump", "major", "--build", "1999"];
        let config = Config {
            version_modifier: VersionModifier {
                mod_type: ModifierType::Major,
                build_metadata: BuildMetadata::from_str("1999").ok(),
                pre_release: None,
            },
            ..Default::default()
        };
        test_config(input, &config);
    }

    #[test]
    fn version_bump_and_pre() {
        let input = vec!["cargo-bump", "2.0.0", "--pre-release", "beta"];
        let config = Config {
            version_modifier: VersionModifier {
                mod_type: ModifierType::Replace(Version::parse("2.0.0").unwrap()),
                build_metadata: None,
                pre_release: Prerelease::from_str("beta").ok(),
            },
            ..Default::default()
        };
        test_config(input, &config);
    }
}
