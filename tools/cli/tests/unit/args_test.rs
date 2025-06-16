#[cfg(test)]
mod cli_args_tests {
    use qudag_cli::cli::Args;
    use clap::Parser;

    #[test]
    fn test_parse_node_start_command() {
        let args = Args::try_parse_from(&["qudag", "node", "start", "--config", "config.toml"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.command, "node");
        assert_eq!(args.subcommand, "start");
        assert_eq!(args.config, Some("config.toml".to_string()));
    }

    #[test]
    fn test_parse_node_stop_command() {
        let args = Args::try_parse_from(&["qudag", "node", "stop"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.command, "node");
        assert_eq!(args.subcommand, "stop");
    }

    #[test]
    fn test_parse_status_command() {
        let args = Args::try_parse_from(&["qudag", "status"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.command, "status");
    }

    #[test]
    fn test_parse_invalid_command() {
        let args = Args::try_parse_from(&["qudag", "invalid"]);
        assert!(args.is_err());
    }

    #[test]
    fn test_parse_help_flag() {
        let args = Args::try_parse_from(&["qudag", "--help"]);
        assert!(args.is_err()); // Help flag exits with error code
    }

    #[test]
    fn test_parse_version_flag() {
        let args = Args::try_parse_from(&["qudag", "--version"]);
        assert!(args.is_err()); // Version flag exits with error code
    }

    #[test]
    fn test_parse_node_start_with_peer_list() {
        let args = Args::try_parse_from(&[
            "qudag", 
            "node", 
            "start", 
            "--config", 
            "config.toml",
            "--peers",
            "127.0.0.1:8000,127.0.0.1:8001"
        ]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.command, "node");
        assert_eq!(args.subcommand, "start");
        assert_eq!(args.peers, Some(vec!["127.0.0.1:8000".to_string(), "127.0.0.1:8001".to_string()]));
    }

    #[test]
    fn test_parse_node_start_with_bind_address() {
        let args = Args::try_parse_from(&[
            "qudag", 
            "node", 
            "start", 
            "--config", 
            "config.toml",
            "--bind",
            "127.0.0.1:9000"
        ]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.command, "node");
        assert_eq!(args.subcommand, "start");
        assert_eq!(args.bind, Some("127.0.0.1:9000".to_string()));
    }

    #[test]
    fn test_parse_node_start_with_log_level() {
        let args = Args::try_parse_from(&[
            "qudag", 
            "node", 
            "start", 
            "--config", 
            "config.toml",
            "--log-level",
            "debug"
        ]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.command, "node");
        assert_eq!(args.subcommand, "start");
        assert_eq!(args.log_level, Some("debug".to_string()));
    }
}