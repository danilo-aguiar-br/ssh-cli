        use super::{map_exit_status, process_exec_message};
        use crate::ssh::scp_wire::{
            classify_scp_message, format_scp_t_line, format_scp_upload_header,
            format_scp_upload_header_with_mode, interpret_scp_status, parse_scp_header,
            parse_scp_t_line, partial_download_path, remote_scp_command, SCP_PARTIAL_SUFFIX,
        };
        use crate::errors::SshCliError;

        #[test]
        fn map_exit_status_normal() {
            assert_eq!(map_exit_status(0), 0);
            assert_eq!(map_exit_status(255), 255);
        }

        #[test]
        fn map_exit_status_overflow_returns_minus_one() {
            assert_eq!(map_exit_status(u32::MAX), -1);
        }

        #[test]
        fn parse_scp_header_valid_returns_mode_and_size() {
            let (mode, size) =
                parse_scp_header("C0644 42 arquivo.txt\n").expect("valid header");
            assert_eq!(mode, 0o644);
            assert_eq!(size, 42);
            let (mode2, _) = parse_scp_header("C0600 1 x\n").expect("600");
            assert_eq!(mode2, 0o600);
        }

        #[test]
        fn parse_scp_header_invalid_returns_error() {
            assert!(parse_scp_header("ERRO").is_err());
            assert!(parse_scp_header("C0644 sem_tamanho").is_err());
            assert!(parse_scp_header("C0644 abc arquivo").is_err());
            assert!(parse_scp_header("Czzzz 1 x\n").is_err());
        }

        #[test]
        fn process_exec_message_handles_stdout_stderr_close() {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut exit_code = None;
            let mut trunc_out = false;
            let mut trunc_err = false;
            let cap = 64 * 1024;

            let deve_parar = process_exec_message(
                russh::ChannelMsg::Data {
                    data: b"stdout".to_vec().into(),
                },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
                cap,
                &mut trunc_out,
                &mut trunc_err,
            );
            assert!(!deve_parar);
            assert_eq!(stdout, b"stdout");
            assert!(!trunc_out);

            let deve_parar = process_exec_message(
                russh::ChannelMsg::ExtendedData {
                    data: b"stderr".to_vec().into(),
                    ext: 1,
                },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
                cap,
                &mut trunc_out,
                &mut trunc_err,
            );
            assert!(!deve_parar);
            assert_eq!(stderr, b"stderr");
            assert!(!trunc_err);

            let _ = process_exec_message(
                russh::ChannelMsg::ExitStatus { exit_status: 17 },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
                cap,
                &mut trunc_out,
                &mut trunc_err,
            );
            assert_eq!(exit_code, Some(17));

            let deve_parar = process_exec_message(
                russh::ChannelMsg::Close,
                &mut stdout,
                &mut stderr,
                &mut exit_code,
                cap,
                &mut trunc_out,
                &mut trunc_err,
            );
            assert!(deve_parar);
        }

        #[test]
        fn process_exec_message_caps_stdout_bytes() {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut exit_code = None;
            let mut trunc_out = false;
            let mut trunc_err = false;

            let _ = process_exec_message(
                russh::ChannelMsg::Data {
                    data: b"abcdef".to_vec().into(),
                },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
                4,
                &mut trunc_out,
                &mut trunc_err,
            );
            assert_eq!(stdout, b"abcd");
            assert!(trunc_out);
            assert!(!trunc_err);

            // Further data is dropped; buffer stays at cap.
            let _ = process_exec_message(
                russh::ChannelMsg::Data {
                    data: b"zzzz".to_vec().into(),
                },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
                4,
                &mut trunc_out,
                &mut trunc_err,
            );
            assert_eq!(stdout, b"abcd");
            assert!(trunc_out);
        }

        #[test]
        fn format_scp_upload_header_expected_format() {
            let header = format_scp_upload_header(123, "arquivo.txt");
            // Wire protocol: real newline (0x0a), NOT the literal '\'+'n' sequence.
            assert_eq!(header, "C0644 123 arquivo.txt\n");
            assert_eq!(header.as_bytes().last().copied(), Some(b'\n'));
            assert!(
                !header.as_bytes().windows(2).any(|w| w == *b"\\n"),
                "header must not contain literal backslash-n"
            );
        }

        #[test]
        fn format_scp_upload_header_uses_basename() {
            let header = format_scp_upload_header(1, "/tmp/dir/nome.bin");
            assert_eq!(header, "C0644 1 nome.bin\n");
        }

        #[test]
        fn interpret_scp_status_ok_and_error() {
            assert!(interpret_scp_status(&[0]).is_ok());
            assert!(interpret_scp_status(&[1, b'f', b'a', b'i', b'l']).is_err());
            assert!(interpret_scp_status(&[]).is_err());
        }

        /// GAP-SSH-IO-010: remote missing → FileNotFound (exit 66).
        #[test]
        fn interpret_scp_status_no_such_file() {
            let mut payload = vec![1u8];
            payload.extend_from_slice(b"scp: /tmp/missing: No such file or directory\n");
            let err = interpret_scp_status(&payload).unwrap_err();
            assert!(
                matches!(err, SshCliError::FileNotFound(_)),
                "esperado FileNotFound, got {err:?}"
            );
            assert_eq!(err.exit_code(), 66);
        }

        #[test]
        fn classificar_mensagem_scp_protocol_permanece_canal() {
            let err = classify_scp_message("SCP: protocol error");
            assert!(matches!(err, SshCliError::ChannelFailed { .. }));
            assert_eq!(err.exit_code(), 74);
            let err2 = classify_scp_message("SCP stderr: Permission denied");
            assert!(matches!(err2, SshCliError::ChannelFailed { .. }));
            assert_eq!(err2.exit_code(), 74);
        }

        #[test]
        fn classificar_mensagem_scp_not_found_e_66() {
            let err = classify_scp_message("SCP stderr: scp: foo: not found");
            assert!(matches!(err, SshCliError::FileNotFound(_)));
            assert_eq!(err.exit_code(), 66);
        }

        #[test]
        fn remote_scp_command_escapa_path_e_usa_p() {
            let cmd = remote_scp_command("-t", std::path::Path::new("/tmp/a b.txt"));
            assert_eq!(cmd, "scp -tp '/tmp/a b.txt'");
            let cmd_f = remote_scp_command("-f", std::path::Path::new("/var/log/a.log"));
            assert_eq!(cmd_f, "scp -fp '/var/log/a.log'");
            // Idempotent if it already contains p.
            assert_eq!(
                remote_scp_command("-fp", std::path::Path::new("/x")),
                "scp -fp '/x'"
            );
        }

        #[test]
        fn format_scp_t_line_format() {
            let t = format_scp_t_line(1_700_000_000, 1_700_000_001);
            assert_eq!(t, "T1700000000 0 1700000001 0\n");
            assert_eq!(t.as_bytes().last().copied(), Some(b'\n'));
        }

        #[test]
        fn parse_scp_t_line_ok() {
            let (m, a) = parse_scp_t_line("T100 0 200 0\n").expect("T ok");
            assert_eq!((m, a), (100, 200));
        }

        #[test]
        fn format_header_with_mode() {
            let h = format_scp_upload_header_with_mode(0o755, 10, "x.sh");
            assert_eq!(h, "C0755 10 x.sh\n");
        }

        #[test]
        fn partial_download_path_suffix() {
            let p = partial_download_path(std::path::Path::new("/tmp/out.bin"));
            assert!(p.to_string_lossy().ends_with(SCP_PARTIAL_SUFFIX));
            assert!(p.to_string_lossy().contains("out.bin"));
        }

        #[test]
        fn process_exec_message_ignores_extended_non_stderr() {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut exit_code = None;
            let mut trunc_out = false;
            let mut trunc_err = false;

            let deve_parar = process_exec_message(
                russh::ChannelMsg::ExtendedData {
                    data: b"nao-e-stderr".to_vec().into(),
                    ext: 2,
                },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
                1024,
                &mut trunc_out,
                &mut trunc_err,
            );

            assert!(!deve_parar);
            assert!(stdout.is_empty());
            assert!(stderr.is_empty());
            assert!(exit_code.is_none());
        }

        #[test]
        fn process_exec_message_handles_exit_signal_and_eof() {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut exit_code = Some(7);
            let mut trunc_out = false;
            let mut trunc_err = false;

            let deve_parar_signal = process_exec_message(
                russh::ChannelMsg::ExitSignal {
                    signal_name: russh::Sig::TERM,
                    core_dumped: false,
                    error_message: "encerrado".to_string(),
                    lang_tag: "pt-BR".to_string(),
                },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
                1024,
                &mut trunc_out,
                &mut trunc_err,
            );

            let deve_parar_eof = process_exec_message(
                russh::ChannelMsg::Eof,
                &mut stdout,
                &mut stderr,
                &mut exit_code,
                1024,
                &mut trunc_out,
                &mut trunc_err,
            );

            assert!(!deve_parar_signal);
            assert!(!deve_parar_eof);
            assert_eq!(exit_code, Some(7));
        }

        #[test]
        fn process_exec_message_ignores_unhandled_variants() {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut exit_code = None;
            let mut trunc_out = false;
            let mut trunc_err = false;

            let deve_parar = process_exec_message(
                russh::ChannelMsg::WindowAdjusted { new_size: 2048 },
                &mut stdout,
                &mut stderr,
                &mut exit_code,
                1024,
                &mut trunc_out,
                &mut trunc_err,
            );

            assert!(!deve_parar);
            assert!(stdout.is_empty());
            assert!(stderr.is_empty());
            assert!(exit_code.is_none());
        }
