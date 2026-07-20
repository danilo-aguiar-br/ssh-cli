// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP-R: CLI unit tests extracted from mod.rs (SRP).
#![allow(clippy::unwrap_used)]

use super::*;
use clap::{CommandFactory, Parser};

    /// Clap invariant: catch developer-definition bugs (rules: CommandFactory::debug_assert).
    #[test]
    fn cli_command_debug_assert() {
        CliArgs::command().debug_assert();
    }

    #[test]
    fn effective_timeout_local_wins_over_global() {
        assert_eq!(effective_timeout(Some(100), Some(999)), Some(100));
        assert_eq!(effective_timeout(None, Some(5000)), Some(5000));
        assert_eq!(effective_timeout(None, None), None);
    }

    #[test]
    fn parser_accepts_global_timeout() {
        let args = CliArgs::try_parse_from([
            "ssh-cli",
            "--timeout",
            "1234",
            "vps",
            "list",
        ])
        .expect("parse");
        assert_eq!(args.timeout, Some(1234));
    }

    #[test]
    fn parse_cli_char_limit_accepts_none_and_decimal() {
        assert_eq!(parse_cli_char_limit("none").unwrap(), 0);
        assert_eq!(parse_cli_char_limit("0").unwrap(), 0);
        assert_eq!(parse_cli_char_limit("4096").unwrap(), 4096);
        assert!(parse_cli_char_limit("nope").is_err());
    }

    #[test]
    fn verbose_conflicts_with_quiet() {
        let err = CliArgs::try_parse_from(["ssh-cli", "-v", "-q", "vps", "path"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn parser_vps_add_max_command_chars_usize() {
        let args = CliArgs::try_parse_from([
            "ssh-cli",
            "vps",
            "add",
            "--name",
            "x",
            "--host",
            "h",
            "--user",
            "u",
            "--password",
            "p",
            "--max-command-chars",
            "2000",
            "--max-output-chars",
            "none",
        ])
        .expect("add limits");
        match args.command {
            Command::Vps {
                action:
                    VpsAction::Add {
                        max_command_chars,
                        max_output_chars,
                        ..
                    },
            } => {
                assert_eq!(max_command_chars, Some(2000));
                assert_eq!(max_output_chars, Some(0));
            }
            _ => panic!("esperado add"),
        }
    }

    #[test]
    fn parser_export_output_pathbuf() {
        let args = CliArgs::try_parse_from([
            "ssh-cli",
            "vps",
            "export",
            "-o",
            "/tmp/hosts.toml",
        ])
        .expect("export");
        match args.command {
            Command::Vps {
                action: VpsAction::Export { output, .. },
            } => {
                assert_eq!(output.as_deref(), Some(std::path::Path::new("/tmp/hosts.toml")));
            }
            _ => panic!("export"),
        }
    }

    #[test]
    fn parser_commands_meta() {
        let args = CliArgs::try_parse_from(["ssh-cli", "commands", "--json"]).expect("commands");
        match args.command {
            Command::Commands { json } => assert!(json),
            _ => panic!("expected commands"),
        }
        let tree = command_tree_json();
        assert_eq!(tree["ok"], true);
        assert_eq!(tree["event"], "commands");
        assert_eq!(tree["bin"], "ssh-cli");
        assert!(tree["tree"]["subcommands"].as_array().unwrap().len() >= 5);
        let names: Vec<&str> = tree["tree"]["subcommands"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|c| c["name"].as_str())
            .collect();
        assert!(
            names.contains(&"locale"),
            "command tree must include locale: {names:?}"
        );
    }

    #[test]
    fn parser_lang_global_accepts_pt_br() {
        let args = CliArgs::try_parse_from(["ssh-cli", "--lang", "pt-BR", "locale"])
            .expect("lang+locale");
        assert_eq!(args.lang.as_deref(), Some("pt-BR"));
        assert!(matches!(args.command, Command::Locale { .. }));
    }

    #[test]
    fn parser_lang_rejects_unsupported() {
        let err = CliArgs::try_parse_from(["ssh-cli", "--lang", "fr-FR", "locale"]);
        assert!(err.is_err(), "unsupported --lang must fail clap validation");
    }

    #[test]
    fn parser_locale_set_and_clear() {
        let set = CliArgs::try_parse_from(["ssh-cli", "locale", "set", "en"]).expect("set");
        match set.command {
            Command::Locale {
                action: Some(LocaleAction::Set { lang }),
                ..
            } => assert_eq!(lang, "en"),
            _ => panic!("expected locale set"),
        }
        let clear = CliArgs::try_parse_from(["ssh-cli", "locale", "clear"]).expect("clear");
        assert!(matches!(
            clear.command,
            Command::Locale {
                action: Some(LocaleAction::Clear),
                ..
            }
        ));
    }

    #[test]
    fn parser_auth_flatten_on_exec() {
        let args = CliArgs::try_parse_from([
            "ssh-cli",
            "exec",
            "prod",
            "true",
            "--key",
            "/tmp/id",
            "--password-stdin",
        ])
        .expect("exec auth");
        match args.command {
            Command::Exec { auth, .. } => {
                assert!(auth.password_stdin);
                assert_eq!(
                    auth.key_path_string().as_deref(),
                    Some("/tmp/id")
                );
            }
            _ => panic!("exec"),
        }
    }

    #[test]
    fn parser_edit_enable_sudo_flag() {
        let args = CliArgs::try_parse_from([
            "ssh-cli",
            "vps",
            "edit",
            "prod",
            "--enable-sudo",
        ])
        .expect("edit enable");
        match args.command {
            Command::Vps {
                action:
                    VpsAction::Edit {
                        enable_sudo,
                        disable_sudo,
                        ..
                    },
            } => {
                assert!(enable_sudo);
                assert!(!disable_sudo);
            }
            _ => panic!("edit"),
        }
    }

    #[test]
    fn manpage_renders_non_empty() {
        let bytes = render_manpage().expect("man");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("ssh-cli"));
        assert!(s.contains(".TH") || s.contains("NAME") || s.len() > 50);
    }

    #[test]
    fn parser_understands_tunnel_with_timeout() {
        let args = CliArgs::try_parse_from([
            "ssh-cli",
            "tunnel",
            "vps-a",
            "8080",
            "127.0.0.1",
            "5432",
            "--timeout-ms",
            "5000",
            "--json",
        ])
        .expect("tunnel");
        match args.command {
            Command::Tunnel {
                timeout_ms,
                local_port,
                json,
                ..
            } => {
                assert_eq!(timeout_ms, 5000);
                assert_eq!(local_port, 8080);
                assert!(json);
            }
            _ => panic!("esperado tunnel"),
        }
    }

    #[test]
    fn parser_vps_add_key() {
        let args = CliArgs::try_parse_from([
            "ssh-cli",
            "vps",
            "add",
            "--name",
            "x",
            "--host",
            "h",
            "--user",
            "u",
            "--key",
            "/tmp/id_ed25519",
        ])
        .expect("add key");
        match args.command {
            Command::Vps {
                action: VpsAction::Add { key, password, .. },
            } => {
                assert_eq!(
                    key.as_ref().map(|p| p.to_string_lossy().into_owned()).as_deref(),
                    Some("/tmp/id_ed25519")
                );
                assert!(password.is_none());
            }
            _ => panic!("esperado add"),
        }
    }

    #[test]
    fn parser_sudo_exec_description() {
        let args = CliArgs::try_parse_from([
            "ssh-cli",
            "sudo-exec",
            "v",
            "id",
            "--description",
            "who am i",
        ])
        .unwrap();
        match args.command {
            Command::SudoExec { description, .. } => {
                assert_eq!(description.as_deref(), Some("who am i"));
            }
            _ => panic!("sudo-exec"),
        }
    }

    #[test]
    fn parser_su_exec() {
        let args = CliArgs::try_parse_from(["ssh-cli", "su-exec", "v", "whoami"]).unwrap();
        assert!(matches!(args.command, Command::SuExec { .. }));
    }

    #[test]
    fn parser_disable_sudo_global() {
        let args =
            CliArgs::try_parse_from(["ssh-cli", "--disable-sudo", "vps", "path"]).unwrap();
        assert!(args.disable_sudo);
    }

    #[test]
    fn parser_doctor() {
        let args = CliArgs::try_parse_from(["ssh-cli", "vps", "doctor", "--json"]).unwrap();
        match args.command {
            Command::Vps {
                action:
                    VpsAction::Doctor {
                        json,
                        probe_ssh,
                        hosts,
                    },
            } => {
                assert!(json);
                assert!(!probe_ssh);
                assert!(hosts.is_none());
            }
            _ => panic!("doctor"),
        }
    }

    #[test]
    fn parser_doctor_probe_ssh() {
        let args =
            CliArgs::try_parse_from(["ssh-cli", "vps", "doctor", "--probe-ssh"]).unwrap();
        match args.command {
            Command::Vps {
                action: VpsAction::Doctor { probe_ssh, hosts, .. },
            } => {
                assert!(probe_ssh);
                assert!(hosts.is_none());
            }
            _ => panic!("doctor --probe-ssh"),
        }
    }

    #[test]
    fn parser_doctor_probe_ssh_hosts() {
        let args = CliArgs::try_parse_from([
            "ssh-cli",
            "vps",
            "doctor",
            "--probe-ssh",
            "--hosts",
            "a,b",
            "--json",
        ])
        .unwrap();
        match args.command {
            Command::Vps {
                action:
                    VpsAction::Doctor {
                        probe_ssh,
                        hosts,
                        json,
                    },
            } => {
                assert!(probe_ssh);
                assert!(json);
                assert_eq!(hosts.as_deref(), Some("a,b"));
            }
            _ => panic!("doctor --probe-ssh --hosts"),
        }
    }

    #[test]
    fn parser_max_concurrency_global() {
        let args = CliArgs::try_parse_from([
            "ssh-cli",
            "--max-concurrency",
            "8",
            "health-check",
            "--all",
        ])
        .unwrap();
        assert_eq!(args.max_concurrency, Some(8));
        assert!(matches!(
            args.command,
            Command::HealthCheck { all: true, .. }
        ));
    }

    #[test]
    fn parser_exec_all_single_positional() {
        let args = CliArgs::try_parse_from(["ssh-cli", "exec", "--all", "uptime"]).unwrap();
        match args.command {
            Command::Exec {
                all,
                hosts,
                target,
                ..
            } => {
                assert!(all);
                assert!(hosts.is_none());
                assert_eq!(target, vec!["uptime".to_string()]);
                let (sel, cmd) = parse_exec_target(all, hosts, None, target, None).unwrap();
                assert_eq!(sel, crate::vps::HostSelection::All);
                assert_eq!(cmd, "uptime");
            }
            _ => panic!("exec --all"),
        }
    }

    fn test_vps(name: &str) -> crate::domain::VpsName {
        crate::domain::VpsName::try_new(name).expect("valid test VpsName")
    }

    #[test]
    fn parse_exec_target_single_host() {
        let (sel, cmd) =
            parse_exec_target(false, None, None, vec!["prod".into(), "hostname".into()], None).unwrap();
        assert_eq!(sel, crate::vps::HostSelection::Single(test_vps("prod")));
        assert_eq!(cmd, "hostname");
    }

    #[test]
    fn parse_exec_target_hosts_dedupes() {
        let (sel, cmd) = parse_exec_target(
            false,
            Some("a,b,a".into()),
            None,
            vec!["uptime".into()],
            None,
        )
        .unwrap();
        assert_eq!(
            sel,
            crate::vps::HostSelection::Named(vec![test_vps("a"), test_vps("b")])
        );
        assert_eq!(cmd, "uptime");
        assert!(sel.is_batch());
    }

    #[test]
    fn parse_scp_target_multi_file_single_host() {
        let plan = parse_scp_target(
            false,
            None,
            vec![
                "prod".into(),
                "a.bin".into(),
                "b.bin".into(),
                "/tmp".into(),
            ],
        )
        .unwrap();
        match plan {
            ScpPathPlan::MultiFile {
                vps,
                sources,
                dest_dir,
            } => {
                assert_eq!(vps, "prod");
                assert_eq!(sources.len(), 2);
                assert_eq!(dest_dir, PathBuf::from("/tmp"));
            }
            other => panic!("expected MultiFile, got {other:?}"),
        }
    }

    #[test]
    fn parse_scp_target_multi_file_with_all_is_cartesian() {
        // G-PAR-48: multi-host × multi-file accepted (bound = host sessions).
        let plan = parse_scp_target(
            true,
            None,
            vec!["a.bin".into(), "b.bin".into(), "/tmp".into()],
        )
        .unwrap();
        match plan {
            ScpPathPlan::MultiHostMultiFile {
                selection,
                sources,
                dest_dir,
            } => {
                assert_eq!(selection, crate::vps::HostSelection::All);
                assert_eq!(sources.len(), 2);
                assert_eq!(dest_dir, PathBuf::from("/tmp"));
            }
            other => panic!("expected MultiHostMultiFile, got {other:?}"),
        }
    }

    #[test]
    fn parse_scp_target_multi_file_with_hosts() {
        let plan = parse_scp_target(
            false,
            Some("x,y".into()),
            vec!["a.bin".into(), "b.bin".into(), "/remote/dir".into()],
        )
        .unwrap();
        match plan {
            ScpPathPlan::MultiHostMultiFile {
                selection,
                sources,
                dest_dir,
            } => {
                assert_eq!(
                    selection,
                    crate::vps::HostSelection::Named(vec![test_vps("x"), test_vps("y")])
                );
                assert_eq!(sources.len(), 2);
                assert_eq!(dest_dir, PathBuf::from("/remote/dir"));
            }
            other => panic!("expected MultiHostMultiFile, got {other:?}"),
        }
    }

    #[test]
    fn parse_scp_target_classic_three() {
        let plan = parse_scp_target(
            false,
            None,
            vec!["prod".into(), "a.bin".into(), "/tmp/a.bin".into()],
        )
        .unwrap();
        match plan {
            ScpPathPlan::Single {
                selection,
                path_a,
                path_b,
            } => {
                assert_eq!(
                    selection,
                    crate::vps::HostSelection::Single(test_vps("prod"))
                );
                assert_eq!(path_a, PathBuf::from("a.bin"));
                assert_eq!(path_b, PathBuf::from("/tmp/a.bin"));
            }
            other => panic!("expected Single, got {other:?}"),
        }
    }

    #[test]
    fn parser_exec_hosts_flag() {
        let args = CliArgs::try_parse_from([
            "ssh-cli",
            "exec",
            "--hosts",
            "a,b",
            "uptime",
        ])
        .unwrap();
        match args.command {
            Command::Exec {
                all,
                hosts,
                target,
                ..
            } => {
                assert!(!all);
                assert_eq!(hosts.as_deref(), Some("a,b"));
                assert_eq!(target, vec!["uptime".to_string()]);
            }
            _ => panic!("exec --hosts"),
        }
    }

    #[test]
    fn parser_all_hosts_conflict() {
        let err = CliArgs::try_parse_from([
            "ssh-cli",
            "exec",
            "--all",
            "--hosts",
            "a",
            "uptime",
        ]);
        assert!(err.is_err());
    }
