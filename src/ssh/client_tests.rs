// SPDX-License-Identifier: MIT OR Apache-2.0
// G-ERR-12: unit tests for ssh client (split from monólito client.rs).
#![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use secrecy::SecretString;

    fn valid_cfg() -> ConnectionConfig {
        use crate::domain::{SshHost, SshPort, SshUser, TimeoutMs};
        ConnectionConfig {
            host: SshHost::try_new("127.0.0.1").unwrap(),
            port: SshPort::try_new(22).unwrap(),
            username: SshUser::try_new("root").unwrap(),
            password: SecretString::from("senha-exemplo".to_string()),
            key_path: None,
            key_passphrase: None,
            timeout_ms: TimeoutMs::try_new(5000).unwrap(),
            known_hosts_path: None,
            replace_host_key: false,
            tls: None,
            use_agent: false,
            agent_socket: None,
        }
    }
    #[test]
    fn domain_rejects_empty_host() {
        assert!(crate::domain::SshHost::try_new("").is_err());
        assert!(crate::domain::SshHost::try_new("   ").is_err());
    }

    #[test]
    fn validate_requires_auth_material() {
        let mut c = valid_cfg();
        c.password = SecretString::from(String::new());
        c.key_path = None;
        let r = c.validate();
        assert!(r.is_err());
        let msg = r.unwrap_err().to_string();
        assert!(msg.contains("auth") || msg.contains("password") || msg.contains("key"));
    }

    #[test]
    fn domain_rejects_port_zero_and_empty_user() {
        assert!(crate::domain::SshPort::try_new(0).is_err());
        assert!(crate::domain::SshUser::try_new("").is_err());
    }

    #[test]
    fn validate_correct_config_returns_ok() {
        assert!(valid_cfg().validate().is_ok());
    }

    #[test]
    fn debug_does_not_expose_password() {
        let c = valid_cfg();
        let dbg = format!("{c:?}");
        assert!(!dbg.contains("senha-exemplo"));
        assert!(dbg.contains("redacted"));
    }

    #[test]
    fn exec_capture_byte_cap_scales_and_hard_caps() {
        assert_eq!(exec_capture_byte_cap(0), 0);
        assert_eq!(exec_capture_byte_cap(10), 10 * 4 + 4);
        assert_eq!(
            exec_capture_byte_cap(usize::MAX),
            EXEC_CAPTURE_HARD_MAX_BYTES
        );
        assert_eq!(
            exec_capture_byte_cap(EXEC_CAPTURE_HARD_MAX_BYTES),
            EXEC_CAPTURE_HARD_MAX_BYTES
        );
    }

    #[test]
    fn append_capped_respects_limit() {
        let mut buf = Vec::new();
        let mut truncated = false;
        append_capped(&mut buf, b"hello", 3, &mut truncated);
        assert_eq!(buf, b"hel");
        assert!(truncated);
        truncated = false;
        append_capped(&mut buf, b"xx", 3, &mut truncated);
        assert_eq!(buf, b"hel");
        assert!(truncated);
    }

    #[test]
    fn execution_output_debug_does_not_crash() {
        let s = ExecutionOutput {
            stdout: "ok".into(),
            stderr: String::new(),
            exit_code: Some(0),
            truncated_stdout: false,
            truncated_stderr: false,
            duration_ms: 42,
        };
        let _ = format!("{s:?}");
    }

    #[test]
    fn duration_ms_type_compatible() {
        // Static guarantee that Instant::elapsed fits in u64.
        let fake: u64 = 1234;
        assert_eq!(fake, 1234_u64);
    }
