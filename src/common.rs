//! Common functionality and types.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use tokio::task::spawn_blocking;

use console::Emoji;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::ErrorKind;
use tokio::fs;

pub static BUILDING: Emoji<'_, '_> = Emoji("📦", "");
pub static SUCCESS: Emoji<'_, '_> = Emoji("✅", "");
pub static ERROR: Emoji<'_, '_> = Emoji("❌", "");
pub static SERVER: Emoji<'_, '_> = Emoji("📡", "");

/// Ensure the given value for `--public-url` is formatted correctly.
pub fn parse_public_url(val: &str) -> String {
    let prefix = if !val.starts_with('/') { "/" } else { "" };
    let suffix = if !val.ends_with('/') { "/" } else { "" };
    format!("{}{}{}", prefix, val, suffix)
}

/// A utility function to recursively copy a directory.
pub async fn copy_dir_recursive(from_dir: PathBuf, to_dir: PathBuf) -> Result<()> {
    if !path_exists(&from_dir).await? {
        return Err(anyhow!("directory can not be copied as it does not exist {:?}", &from_dir));
    }

    spawn_blocking(move || {
        let opts = fs_extra::dir::CopyOptions {
            overwrite: true,
            content_only: true,
            ..Default::default()
        };
        let _ = fs_extra::dir::copy(from_dir, to_dir, &opts).context("error copying directory")?;
        Ok(())
    })
    .await
    .context("error copying directory")?
}

/// A utility function to recursively delete a directory.
///
/// Use this instead of fs::remove_dir_all(...) because of Windows compatibility issues, per
/// advice of https://blog.qwaz.io/chat/issues-of-rusts-remove-dir-all-implementation-on-windows
pub async fn remove_dir_all(from_dir: PathBuf) -> Result<()> {
    if !path_exists(&from_dir).await? {
        return Ok(());
    }
    spawn_blocking(move || {
        ::remove_dir_all::remove_dir_all(from_dir.as_path()).context("error removing directory")?;
        Ok(())
    })
    .await?
}

/// Checks if path exists. In case of error, it is returned back from caller.
/// This is behavior is in contrast to [`std::fs::Path::Exists`]
///
/// Taken from: [tokio#3375 (comment)](https://github.com/tokio-rs/tokio/pull/3375#issuecomment-758612179)
pub async fn path_exists(path: impl AsRef<Path>) -> Result<bool> {
    let exists = fs::metadata(&path)
        .await
        .map(|_| true)
        .or_else(|error| if error.kind() == ErrorKind::NotFound { Ok(false) } else { Err(error) })?;
    Ok(exists)
}

/// Build system spinner.
pub fn spinner() -> ProgressBar {
    let style = ProgressStyle::default_spinner().template("{spinner} {prefix} trunk | {wide_msg}");
    ProgressBar::new_spinner().with_style(style)
}
