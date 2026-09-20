use std::{fs, path::Path};

use color_eyre::{eyre::Context, Result};
use russh::keys::{
    ssh_key::{
        private::{Ed25519Keypair, KeypairData},
        LineEnding,
    },
    PrivateKey,
};

/// Loads the OpenSSH-format host key at `path`, generating and saving a new
/// Ed25519 key when the file does not exist. A stable host key matters
/// because clients pin it in `known_hosts` on first connect.
pub fn load_or_generate(path: &Path) -> Result<PrivateKey> {
    if path.exists() {
        let key = russh::keys::load_secret_key(path, None)
            .wrap_err_with(|| format!("loading host key {}", path.display()))?;
        log::info!("Loaded host key from {}", path.display());
        return Ok(key);
    }

    // Built from a seed rather than `PrivateKey::random` because ssh-key and
    // the rand crate disagree on the rand_core version; 32 random bytes are
    // all an Ed25519 key needs.
    let seed: [u8; 32] = rand::random();
    let key = PrivateKey::new(
        KeypairData::from(Ed25519Keypair::from_seed(&seed)),
        "mothership host key",
    )?;
    let pem = key.to_openssh(LineEnding::LF)?;
    fs::write(path, pem.as_bytes())
        .wrap_err_with(|| format!("saving host key to {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    log::info!("Generated new host key at {}", path.display());
    Ok(key)
}
