//! Unit tests for the `config_io` module.
#![forbid(unsafe_code)]

use super::*;
use secrecy::{ExposeSecret, SecretString};
use tempfile::TempDir;

fn reg_min() -> VpsRecord {
    VpsRecord::test_new(
        "srv",
        "host.example.com",
        2222,
        "admin",
        SecretString::from("pass".to_string()),
        None,
        None,
        Some(60_000),
        Some(1_000),
        Some(50_000),
        None,
        None,
        false,
    )
}

#[test]
fn empty_file_serializes_with_schema() {
    let cfg_file = ConfigFile {
        schema_version: crate::vps::model::CURRENT_SCHEMA_VERSION,
        hosts: BTreeMap::new(),
    };
    let text = toml::to_string(&cfg_file).unwrap();
    assert!(text.contains("schema_version = 3"));
}

#[test]
#[serial_test::serial]
fn atomic_save_roundtrip() {
    let tmp = TempDir::new().unwrap();
    crate::secrets::set_config_dir(Some(tmp.path().to_path_buf()));
    crate::secrets::set_runtime_flags(true, None, false);
    let path = tmp.path().join("config.toml");
    let mut cfg_file = ConfigFile {
        schema_version: 2,
        hosts: BTreeMap::new(),
    };
    cfg_file.hosts.insert("srv".into(), reg_min());
    save(&path, &cfg_file).unwrap();
    let loaded = load(&path).unwrap();
    assert_eq!(loaded.hosts.len(), 1);
    assert_eq!(loaded.hosts["srv"].password.expose_secret(), "pass");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        let lock = path.with_extension("toml.lock");
        if lock.exists() {
            let lm = std::fs::metadata(&lock).unwrap().permissions().mode() & 0o777;
            assert_eq!(lm, 0o600);
        }
    }
    crate::secrets::set_runtime_flags(false, None, false);
    crate::secrets::set_config_dir(None);
}

#[test]
fn resolve_config_path_with_dir_override() {
    let result = resolve_config_path(Some(Path::new("/tmp/test-dir")));
    assert_eq!(result.unwrap(), PathBuf::from("/tmp/test-dir/config.toml"));
}

#[test]
fn resolve_config_path_toml_file_override_keeps_path() {
    let p = Path::new("/tmp/custom-hosts.toml");
    let result = resolve_config_path(Some(p)).unwrap();
    assert_eq!(result, PathBuf::from("/tmp/custom-hosts.toml"));
}

#[test]
fn config_override_shared_without_clone() {
    let owned = PathBuf::from("/tmp/share-me");
    let a = resolve_config_path(Some(owned.as_path())).unwrap();
    let b = winning_layer(Some(owned.as_path())).unwrap();
    assert_eq!(a, PathBuf::from("/tmp/share-me/config.toml"));
    assert_eq!(b.name, "--config-dir");
    assert_eq!(b.path, a);
    assert_eq!(owned, PathBuf::from("/tmp/share-me"));
}

fn contended() -> std::io::Result<()> {
    Err(std::io::ErrorKind::WouldBlock.into())
}

#[test]
fn lock_wait_stops_within_one_slice_of_the_signal() {
    use std::cell::Cell;
    use std::time::Duration;
    let stop_calls = Cell::new(0u32);
    let try_calls = Cell::new(0u32);
    let outcome = wait_for_lock(
        || {
            try_calls.set(try_calls.get() + 1);
            contended()
        },
        || {
            stop_calls.set(stop_calls.get() + 1);
            stop_calls.get() >= 3
        },
        Duration::from_secs(60),
        Duration::from_millis(1),
    )
    .unwrap();
    assert_eq!(outcome, LockWait::Interrupted);
    // The first `true` ends the wait: no call happens after it.
    assert_eq!(stop_calls.get(), 3);
    assert!(try_calls.get() <= 3);
}

#[test]
fn lock_wait_sleeps_in_slices_and_never_passes_the_budget() {
    use std::cell::Cell;
    use std::time::{Duration, Instant};
    let stop_calls = Cell::new(0u32);
    let try_calls = Cell::new(0u32);
    let budget = Duration::from_millis(60);
    let start = Instant::now();
    let outcome = wait_for_lock(
        || {
            try_calls.set(try_calls.get() + 1);
            contended()
        },
        || {
            stop_calls.set(stop_calls.get() + 1);
            false
        },
        budget,
        Duration::from_millis(2),
    )
    .unwrap();
    let waited = start.elapsed();
    assert_eq!(outcome, LockWait::DeadlineExpired);
    assert!(waited >= budget, "returned early: {waited:?}");
    // Generous bound: the last sleep is clamped to the remaining budget.
    assert!(waited < budget + Duration::from_millis(500), "{waited:?}");
    // One stop check per slice, not one per whole backoff delay.
    assert!(stop_calls.get() >= try_calls.get() + 5);
}

#[test]
fn lock_wait_acquires_after_contention() {
    use std::cell::Cell;
    use std::time::Duration;
    let tries = Cell::new(0u32);
    let outcome = wait_for_lock(
        || {
            tries.set(tries.get() + 1);
            if tries.get() < 3 {
                contended()
            } else {
                Ok(())
            }
        },
        || false,
        Duration::from_secs(60),
        Duration::from_millis(1),
    )
    .unwrap();
    assert_eq!(outcome, LockWait::Acquired);
    assert_eq!(tries.get(), 3);
}

#[test]
fn lock_wait_stop_beats_an_expired_budget() {
    use std::time::Duration;
    let outcome =
        wait_for_lock(contended, || true, Duration::ZERO, Duration::from_millis(1)).unwrap();
    assert_eq!(outcome, LockWait::Interrupted);
}
