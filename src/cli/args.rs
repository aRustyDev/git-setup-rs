use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "git-setup")]
#[command(about = "Manage Git profiles with 1Password integration")]
#[command(version)]
#[command(disable_version_flag = true)]
pub struct Args {
    /// Profile name to apply
    pub profile: Option<String>,

    /// Apply configuration globally (~/.gitconfig)
    #[arg(long, conflicts_with = "system")]
    pub global: bool,

    /// Apply configuration system-wide (/etc/gitconfig)
    #[arg(long)]
    pub system: bool,

    /// Enable verbose output
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// Suppress output
    #[arg(long, short = 'q')]
    pub quiet: bool,

    /// Add a new profile
    #[arg(long, short = 'a')]
    pub add: Option<String>,

    /// Delete a profile
    #[arg(long, short = 'd')]
    pub delete: Option<String>,

    /// Edit a profile
    #[arg(long, short = 'e')]
    pub edit: Option<String>,

    /// List all profiles
    #[arg(long, short = 'l')]
    pub list: bool,

    /// Import profiles from 1Password (agent.toml)
    #[arg(long, short = 'i')]
    pub import: bool,

    /// Output format
    #[arg(long, short = 'o', default_value = "tabular")]
    pub output: OutputFormat,

    /// Configuration file to use
    #[arg(long, short = 'f')]
    pub file: Option<String>,

    /// Show version information
    #[arg(long)]
    pub version: bool,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    Yaml,
    Toml,
    Csv,
    Tabular,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_parse_profile_name() {
        let args = Args::try_parse_from(&["git-setup", "work"]).unwrap();
        assert_eq!(args.profile, Some("work".to_string()));
    }

    #[test]
    fn test_global_system_conflict() {
        let result = Args::try_parse_from(&["git-setup", "--global", "--system"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_output_format() {
        let args = Args::try_parse_from(&["git-setup", "--list"]).unwrap();
        assert!(matches!(args.output, OutputFormat::Tabular));
    }

    #[test]
    fn test_verbose_flag() {
        let args = Args::try_parse_from(&["git-setup", "--verbose"]).unwrap();
        assert!(args.verbose);

        let args = Args::try_parse_from(&["git-setup", "-v"]).unwrap();
        assert!(args.verbose);
    }

    #[test]
    fn test_quiet_flag() {
        let args = Args::try_parse_from(&["git-setup", "--quiet"]).unwrap();
        assert!(args.quiet);

        let args = Args::try_parse_from(&["git-setup", "-q"]).unwrap();
        assert!(args.quiet);
    }

    #[test]
    fn test_add_profile() {
        let args = Args::try_parse_from(&["git-setup", "--add", "new-profile"]).unwrap();
        assert_eq!(args.add, Some("new-profile".to_string()));

        let args = Args::try_parse_from(&["git-setup", "-a", "another-profile"]).unwrap();
        assert_eq!(args.add, Some("another-profile".to_string()));
    }

    #[test]
    fn test_delete_profile() {
        let args = Args::try_parse_from(&["git-setup", "--delete", "old-profile"]).unwrap();
        assert_eq!(args.delete, Some("old-profile".to_string()));

        let args = Args::try_parse_from(&["git-setup", "-d", "another-old-profile"]).unwrap();
        assert_eq!(args.delete, Some("another-old-profile".to_string()));
    }

    #[test]
    fn test_edit_profile() {
        let args = Args::try_parse_from(&["git-setup", "--edit", "edit-profile"]).unwrap();
        assert_eq!(args.edit, Some("edit-profile".to_string()));

        let args = Args::try_parse_from(&["git-setup", "-e", "another-edit-profile"]).unwrap();
        assert_eq!(args.edit, Some("another-edit-profile".to_string()));
    }

    #[test]
    fn test_list_flag() {
        let args = Args::try_parse_from(&["git-setup", "--list"]).unwrap();
        assert!(args.list);

        let args = Args::try_parse_from(&["git-setup", "-l"]).unwrap();
        assert!(args.list);
    }

    #[test]
    fn test_import_flag() {
        let args = Args::try_parse_from(&["git-setup", "--import"]).unwrap();
        assert!(args.import);

        let args = Args::try_parse_from(&["git-setup", "-i"]).unwrap();
        assert!(args.import);
    }

    #[test]
    fn test_output_format_options() {
        let args = Args::try_parse_from(&["git-setup", "--output", "json"]).unwrap();
        assert!(matches!(args.output, OutputFormat::Json));

        let args = Args::try_parse_from(&["git-setup", "-o", "yaml"]).unwrap();
        assert!(matches!(args.output, OutputFormat::Yaml));

        let args = Args::try_parse_from(&["git-setup", "--output", "toml"]).unwrap();
        assert!(matches!(args.output, OutputFormat::Toml));

        let args = Args::try_parse_from(&["git-setup", "--output", "csv"]).unwrap();
        assert!(matches!(args.output, OutputFormat::Csv));

        let args = Args::try_parse_from(&["git-setup", "--output", "tabular"]).unwrap();
        assert!(matches!(args.output, OutputFormat::Tabular));
    }

    #[test]
    fn test_config_file() {
        let args = Args::try_parse_from(&["git-setup", "--file", "/path/to/config.toml"]).unwrap();
        assert_eq!(args.file, Some("/path/to/config.toml".to_string()));

        let args = Args::try_parse_from(&["git-setup", "-f", "config.toml"]).unwrap();
        assert_eq!(args.file, Some("config.toml".to_string()));
    }

    #[test]
    fn test_version_flag() {
        let args = Args::try_parse_from(&["git-setup", "--version"]).unwrap();
        assert!(args.version);
    }

    #[test]
    fn test_global_flag() {
        let args = Args::try_parse_from(&["git-setup", "--global"]).unwrap();
        assert!(args.global);
        assert!(!args.system);
    }

    #[test]
    fn test_system_flag() {
        let args = Args::try_parse_from(&["git-setup", "--system"]).unwrap();
        assert!(args.system);
        assert!(!args.global);
    }

    #[test]
    fn test_help_generates() {
        let cmd = Args::command();
        let help = cmd.try_get_matches_from(&["git-setup", "--help"]);
        // This should fail (expected behavior for --help), but the command should be valid
        assert!(help.is_err());

        // Test that the command factory works
        let _cmd = Args::command();
    }

    #[test]
    fn test_no_args_defaults() {
        let args = Args::try_parse_from(&["git-setup"]).unwrap();
        assert_eq!(args.profile, None);
        assert!(!args.global);
        assert!(!args.system);
        assert!(!args.verbose);
        assert!(!args.quiet);
        assert_eq!(args.add, None);
        assert_eq!(args.delete, None);
        assert_eq!(args.edit, None);
        assert!(!args.list);
        assert!(!args.import);
        assert!(matches!(args.output, OutputFormat::Tabular));
        assert_eq!(args.file, None);
        assert!(!args.version);
    }

    #[test]
    fn test_multiple_compatible_flags() {
        let args = Args::try_parse_from(&["git-setup", "--verbose", "--list", "--output", "json"])
            .unwrap();
        assert!(args.verbose);
        assert!(args.list);
        assert!(matches!(args.output, OutputFormat::Json));
    }
}
