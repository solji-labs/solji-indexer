use anyhow::{bail, Result};
use base64::{engine::general_purpose, Engine as _};
use borsh::BorshDeserialize;
use sha2::{Digest, Sha256};
// get event discriminator
pub fn event_disc(name: &str) -> [u8; 8] {
    let mut h = Sha256::new();
    h.update(format!("event:{name}"));
    let b = h.finalize();
    let mut d = [0u8; 8];
    d.copy_from_slice(&b[..8]);
    d
}

pub fn decode_anchor_event<T: BorshDeserialize>(logs: &[String], disc: &[u8; 8]) -> Option<T> {
    for line in logs {
        if let Some(b64) = line.strip_prefix("Program data: ") {
            let bytes = general_purpose::STANDARD.decode(b64).ok()?;
            if bytes.len() >= 8 && &bytes[..8] == disc {
                let mut s = &bytes[8..];
                return T::deserialize(&mut s).ok();
            }
        }
    }
    None
}

pub fn decode_anchor_account<T: BorshDeserialize>(
    data: &[u8],
    expected_disc: &[u8; 8],
) -> Result<T> {
    if data.len() < 8 {
        bail!("account data too short: {}", data.len());
    }
    if &data[..8] != expected_disc {
        bail!(
            "discriminator mismatch: got {:02x?}, expect {:02x?}",
            &data[..8],
            expected_disc
        );
    }
    let mut s = &data[8..];
    Ok(T::deserialize(&mut s)?)
}

pub fn account_disc(name: &str) -> [u8; 8] {
    let mut h = Sha256::new();
    h.update(format!("account:{name}"));
    let b = h.finalize();
    let mut d = [0u8; 8];
    d.copy_from_slice(&b[..8]);
    d
}
