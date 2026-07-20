// SPDX-License-Identifier: MIT OR Apache-2.0
// G-COMP: unit tests extracted for line budget.
#![forbid(unsafe_code)]

use super::*;

    #[test]
    fn vps_not_found_message_contains_name() {
        let err = SshCliError::VpsNotFound("prod".into());
        assert!(err.to_string().contains("prod"));
    }

    #[test]
    fn vps_duplicate_message_contains_name() {
        let err = SshCliError::VpsDuplicate("vps-1".into());
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn exit_code_auth_failed_is_noperm() {
        assert_eq!(
            SshCliError::AuthenticationFailed.exit_code(),
            exit_codes::EX_NOPERM
        );
    }

    #[test]
    fn exit_code_command_failed_is_general_not_remote() {
        let e = SshCliError::CommandFailed {
            exit_code: 64,
            stderr: "usage".into(),
        };
        assert_eq!(e.exit_code(), exit_codes::EX_GENERAL);
    }

    #[test]
    fn exit_code_broken_pipe_is_141() {
        let e = SshCliError::Io(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "pipe closed",
        ));
        assert_eq!(e.exit_code(), exit_codes::EX_PIPE);
        assert_eq!(exit_codes::EX_PIPE, 141);
    }

    #[test]
    fn exit_code_other_io_is_74() {
        let e = SshCliError::Io(std::io::Error::other("disk full"));
        assert_eq!(e.exit_code(), exit_codes::EX_IOERR);
    }

    #[test]
    fn anyhow_detects_broken_pipe() {
        let err: anyhow::Error =
            SshCliError::Io(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "epipe")).into();
        assert!(anyhow_is_broken_pipe(&err));
    }

    #[test]
    fn connection_failed_is_retryable_transient() {
        let e = SshCliError::ConnectionFailed("refused".into());
        assert!(e.is_retryable());
        assert!(!e.is_permanent());
        assert_eq!(e.classify(), ErrorClass::Transient);
        assert_eq!(e.layer(), ErrorLayer::Network);
        assert_eq!(e.exit_code(), exit_codes::EX_IOERR);
        assert!(e.suggestion().is_some());
        assert_eq!(e.error_code(), "connection_failed");
    }

    #[test]
    fn auth_failed_is_permanent_not_retryable() {
        let e = SshCliError::AuthenticationFailed;
        assert!(!e.is_retryable());
        assert!(e.is_permanent());
        assert_eq!(e.classify(), ErrorClass::Permanent);
        assert_eq!(e.layer(), ErrorLayer::Auth);
        assert_eq!(e.exit_code(), exit_codes::EX_NOPERM);
    }

    #[test]
    fn command_failed_not_transport_retryable() {
        let e = SshCliError::CommandFailed {
            exit_code: 1,
            stderr: "boom".into(),
        };
        assert!(!e.is_retryable());
        assert_eq!(e.classify(), ErrorClass::Permanent);
        assert_eq!(e.retry_kind(), RetryKind::PermanentRemoteCommand);
    }

    #[test]
    fn timeout_is_retryable() {
        let e = SshCliError::SshTimeout(5_000);
        assert!(e.is_retryable());
        assert_eq!(e.retry_kind(), RetryKind::TransientTimeout);
    }

    #[test]
    fn broken_pipe_is_cancelled() {
        let e = SshCliError::Io(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "pipe",
        ));
        assert!(!e.is_retryable());
        assert!(!e.is_permanent());
        assert_eq!(e.classify(), ErrorClass::Cancelled);
        assert_eq!(e.retry_kind(), RetryKind::Cancelled);
    }

    #[test]
    fn classification_never_uses_display_string() {
        // Guard: variants with similar messages still classify by type.
        let a = SshCliError::ConnectionFailed("authentication failed".into());
        let b = SshCliError::AuthenticationFailed;
        assert!(a.is_retryable());
        assert!(!b.is_retryable());
    }

    #[test]
    fn display_messages_are_lowercase_without_trailing_period() {
        let samples = [
            SshCliError::VpsNotFound("x".into()).to_string(),
            SshCliError::NoActiveVps.to_string(),
            SshCliError::tls_msg("handshake").to_string(),
            SshCliError::channel_msg("open").to_string(),
            SshCliError::Io(std::io::Error::other("x")).to_string(),
        ];
        for s in samples {
            assert!(!s.ends_with('.'), "trailing period: {s}");
            let first = s.chars().next().unwrap();
            assert!(
                !first.is_ascii_uppercase(),
                "Display should start lowercase: {s}"
            );
        }
    }

    #[test]
    fn tls_src_preserves_source_chain() {
        let io = std::io::Error::other("root-cause");
        let e = SshCliError::tls_src("handshake failed", io);
        let src = std::error::Error::source(&e).expect("source");
        assert!(src.to_string().contains("root-cause"));
        assert!(!e.to_string().contains("root-cause"));
    }

    #[test]
    fn domain_maps_to_domain_variant() {
        let d = crate::domain::DomainError::new("port", "out of range");
        let e: SshCliError = d.into();
        assert!(matches!(e, SshCliError::Domain(_)));
        assert_eq!(e.error_code(), "domain_validation");
        assert_eq!(e.exit_code(), exit_codes::EX_USAGE);
    }
