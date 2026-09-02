//! Ed25519 signing with the byte layout used by existing nremote keys.
//!
//! Secret keys are stored as `seed || public_key` (64 bytes), and signed
//! messages are encoded as `signature || message`. These are the layouts used
//! by the previous signing implementation, so persisted server keys and every
//! client already in the field remain valid.

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use std::{convert::TryInto, ops::Deref};

pub const PUBLICKEYBYTES: usize = 32;
pub const SECRETKEYBYTES: usize = 64;
pub const SIGNATUREBYTES: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifyError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PublicKey(pub [u8; PUBLICKEYBYTES]);

#[derive(Clone)]
pub struct SecretKey(pub [u8; SECRETKEYBYTES]);

impl PublicKey {
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        let bytes: [u8; PUBLICKEYBYTES] = bytes.try_into().ok()?;
        VerifyingKey::from_bytes(&bytes).ok()?;
        Some(Self(bytes))
    }
}

impl Deref for PublicKey {
    type Target = [u8; PUBLICKEYBYTES];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl SecretKey {
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        let bytes: [u8; SECRETKEYBYTES] = bytes.try_into().ok()?;
        SigningKey::from_keypair_bytes(&bytes).ok()?;
        Some(Self(bytes))
    }
}

impl Deref for SecretKey {
    type Target = [u8; SECRETKEYBYTES];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for SecretKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Drop for SecretKey {
    fn drop(&mut self) {
        self.0.fill(0);
    }
}

pub fn gen_keypair() -> (PublicKey, SecretKey) {
    let seed = rand::random::<[u8; 32]>();
    let signing_key = SigningKey::from_bytes(&seed);
    let public_key = PublicKey(signing_key.verifying_key().to_bytes());
    let secret_key = SecretKey(signing_key.to_keypair_bytes());
    (public_key, secret_key)
}

pub fn sign_detached(message: &[u8], secret_key: &SecretKey) -> Signature {
    // A SecretKey created through `from_slice` or `gen_keypair` has already
    // proved that its public half matches its seed. Reconstructing from the
    // seed keeps signing infallible, matching the established API.
    let seed: [u8; 32] = secret_key.0[..32]
        .try_into()
        .expect("the seed is a fixed-size slice");
    SigningKey::from_bytes(&seed).sign(message)
}

pub fn sign(message: &[u8], secret_key: &SecretKey) -> Vec<u8> {
    let signature = sign_detached(message, secret_key);
    let mut signed = Vec::with_capacity(SIGNATUREBYTES + message.len());
    signed.extend_from_slice(&signature.to_bytes());
    signed.extend_from_slice(message);
    signed
}

pub fn verify(signed: &[u8], public_key: &PublicKey) -> Result<Vec<u8>, VerifyError> {
    if signed.len() < SIGNATUREBYTES {
        return Err(VerifyError);
    }
    let verifying_key = VerifyingKey::from_bytes(&public_key.0).map_err(|_| VerifyError)?;
    let signature = Signature::from_slice(&signed[..SIGNATUREBYTES]).map_err(|_| VerifyError)?;
    let message = &signed[SIGNATUREBYTES..];
    verifying_key
        .verify_strict(message, &signature)
        .map_err(|_| VerifyError)?;
    Ok(message.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode_hex<const N: usize>(value: &str) -> [u8; N] {
        assert_eq!(value.len(), N * 2);
        let mut output = [0u8; N];
        for (index, byte) in output.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).unwrap();
        }
        output
    }

    #[test]
    fn rfc8032_vector_preserves_existing_key_and_signed_message_layout() {
        let seed = decode_hex::<32>(
            "9d61b19deffd5a60ba844af492ec2cc4\
             4449c5697b326919703bac031cae7f60",
        );
        let public = decode_hex::<32>(
            "d75a980182b10ab7d54bfed3c964073a\
             0ee172f3daa62325af021a68f707511a",
        );
        let expected_signature = decode_hex::<64>(
            "e5564300c360ac729086e2cc806e828a\
             84877f1eb8e5d974d873e06522490155\
             5fb8821590a33bacc61e39701cf9b46b\
             d25bf5f0595bbe24655141438e7a100b",
        );

        let mut keypair = [0u8; SECRETKEYBYTES];
        keypair[..32].copy_from_slice(&seed);
        keypair[32..].copy_from_slice(&public);
        let secret_key = SecretKey::from_slice(&keypair).expect("RFC keypair");
        let public_key = PublicKey::from_slice(&public).expect("RFC public key");

        let signed = sign(&[], &secret_key);
        assert_eq!(signed, expected_signature);
        assert_eq!(verify(&signed, &public_key), Ok(Vec::new()));
    }

    #[test]
    fn rejects_mismatched_stored_keypair_and_modified_signature() {
        let (public_key, secret_key) = gen_keypair();
        let mut mismatched = secret_key.0.clone();
        mismatched[63] ^= 1;
        assert!(SecretKey::from_slice(&mismatched).is_none());

        let mut signed = sign(b"message", &secret_key);
        signed[0] ^= 1;
        assert!(verify(&signed, &public_key).is_err());
    }
}
