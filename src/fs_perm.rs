// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]
//! Unix secret file/dir modes — single source (G-AUD-24 / no hardcode drift).

use std::path::Path;

use crate::constants::{SECRET_DIR_MODE_UNIX, SECRET_FILE_MODE_UNIX};
use crate::errors::{SshCliError, SshCliResult};

/// Sets secret-file mode (`0o600`) on Unix; no-op on other targets.
pub fn set_secret_file_mode(path: &Path) -> SshCliResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)
            .map_err(SshCliError::Io)?
            .permissions();
        perms.set_mode(SECRET_FILE_MODE_UNIX);
        std::fs::set_permissions(path, perms).map_err(SshCliError::Io)?;
    }
    let _ = path;
    Ok(())
}

/// Sets secret-dir mode (`0o700`) on Unix; no-op on other targets.
pub fn set_secret_dir_mode(path: &Path) -> SshCliResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)
            .map_err(SshCliError::Io)?
            .permissions();
        perms.set_mode(SECRET_DIR_MODE_UNIX);
        std::fs::set_permissions(path, perms).map_err(SshCliError::Io)?;
    }
    let _ = path;
    Ok(())
}

/// Compile-time alias for call sites that need the raw secret-file mode integer.
#[must_use]
pub const fn secret_file_mode() -> u32 {
    SECRET_FILE_MODE_UNIX
}

/// Compile-time alias for call sites that need the raw secret-dir mode integer.
#[must_use]
pub const fn secret_dir_mode() -> u32 {
    SECRET_DIR_MODE_UNIX
}
