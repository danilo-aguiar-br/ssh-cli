// SPDX-License-Identifier: MIT OR Apache-2.0
// G-SECDEV-05: pure module — no `unsafe` permitted (crate root allows only OS FFI / test env).
#![forbid(unsafe_code)]
//! Only module allowed to emit stdout for VPS CRUD.
//!
//! Centralizes all CRUD formatting for text and JSON emission.
//!
//! Logs (tracing) go to stderr, managed by `tracing-subscriber`.
//!
//! # JSON wire
//!
//! Agent JSON is **compact** single-root RFC 8259 (see [`crate::json_wire`]).
//! Pretty-print is intentionally not used on the machine path.

mod batch;
mod emit;
mod json;
mod text;

pub use batch::{
    print_exec_batch, print_health_batch, print_scp_batch, print_sftp_batch, print_sftp_fs_op_json,
    print_sftp_list_json, print_sftp_stat_json, print_sftp_transfer_json, print_transfer_json,
    print_tunnel_listening_json,
};
pub use emit::{
    emit_success, emit_success_fmt, is_quiet, print_error, print_error_envelope, print_error_fmt,
    print_human_banner, print_json_value, print_success, print_success_fmt, print_warning,
    print_warning_fmt, set_json_errors, set_quiet, wants_json_errors, write_line, write_line_fmt,
    write_line_to, write_line_to_fmt, write_lines, write_stderr_fmt, write_stderr_line,
    write_stderr_line_to, write_stderr_line_to_fmt,
};
pub(crate) use emit::report_json_serialize_error;
pub use json::{
    export_envelope_json, export_hosts_to_json, print_details_json, print_execution_output_json,
    print_health_check_json, print_list_json, record_to_masked_json,
};
pub use text::{
    print_details_text, print_doctor_text, print_execution_output, print_health_check,
    print_list_text,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssh::ExecutionOutput;
    use crate::vps::model::VpsRecord;
    use secrecy::SecretString;

    fn registro_teste() -> VpsRecord {
        VpsRecord::test_new(
            "vps-teste",
            "1.2.3.4",
            22,
            "root",
            SecretString::from("senha-super-secreta".to_string()),
            None,
            None,
            Some(5000),
            Some(1000),
            Some(1000),
            Some(SecretString::from("sudo-password-longa-aqui".to_string())),
            None,
            false,
        )
    }

    #[test]
    fn masked_json_contains_required_fields() {
        let r = registro_teste();
        let m = record_to_masked_json(&r);
        let json = serde_json::to_value(&m).unwrap();
        assert_eq!(json["name"], "vps-teste");
        assert_eq!(json["host"], "1.2.3.4");
        assert_eq!(json["port"], 22);
        assert_eq!(json["user"], "root");
        assert_eq!(json["password"].as_str().unwrap(), "***");
        assert_eq!(json["sudo_password"].as_str().unwrap(), "***");
        assert!(json["su_password"].is_null());
        assert_eq!(json["timeout_ms"], 5000);
        assert_eq!(json["max_command_chars"], 1000);
        assert_eq!(json["max_output_chars"], 1000);
        assert_eq!(json["schema_version"], 3);
    }

    #[test]
    fn masked_json_sudo_null_when_unset() {
        let mut r = registro_teste();
        r.sudo_password = None;
        let json = serde_json::to_value(record_to_masked_json(&r)).unwrap();
        assert!(json["sudo_password"].is_null());
    }

    #[test]
    fn masked_json_su_password_present() {
        let mut r = registro_teste();
        r.su_password = Some(SecretString::from("senha-su-muito-longa-aqui".to_string()));
        let json = serde_json::to_value(record_to_masked_json(&r)).unwrap();
        assert_eq!(json["su_password"].as_str().unwrap(), "***");
    }

    #[test]
    fn masked_json_password_null_when_empty() {
        let mut r = registro_teste();
        r.password = SecretString::from(String::new());
        let json = serde_json::to_value(record_to_masked_json(&r)).unwrap();
        assert!(json["password"].is_null());
    }

    #[test]
    fn write_line_ok() {
        let result = write_line("write test");
        assert!(result.is_ok());
    }

    #[test]
    fn write_line_special_chars() {
        let result = write_line("line with \t tab and \"quotes\"");
        assert!(result.is_ok());
    }

    #[test]
    fn execution_output_fully_formatted() {
        let output = ExecutionOutput {
            stdout: "output do comando".to_string(),
            stderr: "command error".to_string(),
            exit_code: Some(0),
            truncated_stdout: false,
            truncated_stderr: false,
            duration_ms: 150,
        };
        let result = write_line_fmt(format_args!(
            "stdout: {}, stderr: {}, exit: {:?}",
            output.stdout, output.stderr, output.exit_code
        ));
        assert!(result.is_ok());
    }

    #[test]
    fn print_warning_fmt_composes_prefix_without_owned_string() {
        // Smoke: must not panic; prefix composition uses Arguments: Display.
        print_warning_fmt(format_args!("timeout={t}", t = 5u64));
    }

    #[test]
    fn execution_output_without_exit_code() {
        let output = ExecutionOutput {
            stdout: "".to_string(),
            stderr: "".to_string(),
            exit_code: None,
            truncated_stdout: false,
            truncated_stderr: false,
            duration_ms: 0,
        };
        let code_str = output
            .exit_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "N/A".to_string());
        assert_eq!(code_str, "N/A");
    }

    #[test]
    fn vps_record_debug_does_not_expose_password() {
        let r = registro_teste();
        let json_str = serde_json::to_string(&record_to_masked_json(&r)).unwrap();
        assert!(!json_str.contains("senha-super-secreta"));
        assert!(!json_str.contains("sudo-password-longa-aqui"));
        assert!(!json_str.contains('\n'), "agent wire must be compact");
    }

    #[test]
    fn execution_output_truncated_shows_warning() {
        let output = ExecutionOutput {
            stdout: "output".to_string(),
            stderr: "error".to_string(),
            exit_code: Some(1),
            truncated_stdout: true,
            truncated_stderr: true,
            duration_ms: 100,
        };
        assert!(output.truncated_stdout);
        assert!(output.truncated_stderr);
    }

    #[test]
    fn execution_output_numeric_exit_code() {
        let output = ExecutionOutput {
            stdout: "".to_string(),
            stderr: "".to_string(),
            exit_code: Some(127),
            truncated_stdout: false,
            truncated_stderr: false,
            duration_ms: 0,
        };
        let code_str = output
            .exit_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "N/A".to_string());
        assert_eq!(code_str, "127");
    }

    #[test]
    fn write_line_empty_string() {
        let result = write_line("");
        assert!(result.is_ok());
    }

    #[test]
    fn write_line_brazilian_unicode() {
        let result = write_line("ação você está Itaú");
        assert!(result.is_ok());
    }

    #[test]
    fn write_line_with_emojis() {
        let result = write_line("texto com 🚀 e 🔐");
        assert!(result.is_ok());
    }

    #[test]
    fn write_line_with_newlines() {
        let result = write_line("linha1\nlinha2\nlinha3");
        assert!(result.is_ok());
    }

    #[test]
    fn write_line_long_text() {
        let long_text = "a".repeat(10000);
        let result = write_line(&long_text);
        assert!(result.is_ok());
    }

    #[test]
    fn masked_json_short_password_asterisks() {
        let mut r = registro_teste();
        r.password = SecretString::from("curta".to_string());
        let json = serde_json::to_value(record_to_masked_json(&r)).unwrap();
        let password_str = json["password"].as_str().unwrap();
        assert_eq!(password_str, "***");
    }

    #[test]
    fn masked_json_with_sudo_and_su_set() {
        let mut r = registro_teste();
        r.sudo_password = Some(SecretString::from("sudo-pass-longa-aqui".to_string()));
        r.su_password = Some(SecretString::from("su-pass-longa-aqui".to_string()));
        let json = serde_json::to_value(record_to_masked_json(&r)).unwrap();
        assert!(!json["sudo_password"].is_null());
        assert!(!json["su_password"].is_null());
        assert_eq!(json["sudo_password"].as_str().unwrap(), "***");
        assert_eq!(json["su_password"].as_str().unwrap(), "***");
    }

    #[test]
    fn write_line_to_appends_lf_and_flushes() {
        use std::io::Cursor;
        let mut buf = Cursor::new(Vec::new());
        write_line_to(&mut buf, "agent-ok").expect("write");
        assert_eq!(String::from_utf8(buf.into_inner()).unwrap(), "agent-ok\n");
    }

    #[test]
    fn write_line_to_fmt_avoids_owned_string() {
        use std::io::Cursor;
        let mut buf = Cursor::new(Vec::new());
        let port = 22u16;
        write_line_to_fmt(&mut buf, format_args!("port={port}")).expect("write_fmt");
        assert_eq!(String::from_utf8(buf.into_inner()).unwrap(), "port=22\n");
    }

    #[test]
    fn write_stderr_line_to_fmt_treats_broken_pipe_as_ok() {
        use std::io::{self, Write};

        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        write_stderr_line_to_fmt(&mut Broken, format_args!("x"))
            .expect("EPIPE is ok on stderr fmt path");
    }

    #[test]
    fn write_stderr_line_to_treats_broken_pipe_as_ok() {
        use std::io::{self, Write};

        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        write_stderr_line_to(&mut Broken, "x").expect("EPIPE is ok on stderr path");
    }

    #[test]
    fn execution_output_full_formatting() {
        let output = ExecutionOutput {
            stdout: "comando executado".to_string(),
            stderr: "aviso harmless".to_string(),
            exit_code: Some(0),
            truncated_stdout: false,
            truncated_stderr: false,
            duration_ms: 1000,
        };
        assert_eq!(output.stdout, "comando executado");
        assert_eq!(output.stderr, "aviso harmless");
        assert_eq!(output.exit_code, Some(0));
        assert_eq!(output.duration_ms, 1000);
        assert!(!output.truncated_stdout);
        assert!(!output.truncated_stderr);
    }

    #[test]
    fn execution_output_without_stderr() {
        let output = ExecutionOutput {
            stdout: "ok".to_string(),
            stderr: String::new(),
            exit_code: Some(0),
            truncated_stdout: false,
            truncated_stderr: false,
            duration_ms: 50,
        };
        assert!(output.stderr.is_empty());
    }

    #[test]
    fn execution_output_signal_instead_of_exit() {
        let output = ExecutionOutput {
            stdout: String::new(),
            stderr: "signal received".to_string(),
            exit_code: None,
            truncated_stdout: false,
            truncated_stderr: false,
            duration_ms: 5000,
        };
        assert!(output.exit_code.is_none());
    }

    #[test]
    fn execution_output_json_required_fields() {
        let output = ExecutionOutput {
            stdout: "output".to_string(),
            stderr: "error".to_string(),
            exit_code: Some(0),
            truncated_stdout: false,
            truncated_stderr: false,
            duration_ms: 100,
        };
        print_execution_output_json(&output).expect("json print in unit test");
    }
}
