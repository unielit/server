use ring::aead::Aad;
use ring::aead::BoundKey;
use ring::aead::Nonce;
use ring::aead::NonceSequence;
use ring::aead::OpeningKey;
use ring::aead::SealingKey;
use ring::aead::UnboundKey;
use ring::aead::AES_256_GCM;
use ring::aead::NONCE_LEN;
use ring::error::Unspecified;
use ring::rand::SecureRandom;
use ring::rand::SystemRandom;

use std::env;

pub struct EncryptResponse {
    pub cypher: Vec<u8>,
    pub nonce: [u8; NONCE_LEN],
}

struct RandomNonceSequence {
    nonce: [u8; NONCE_LEN],
    rand: SystemRandom,
} 

impl RandomNonceSequence {
    fn new() -> Result<Self, Unspecified> {
        let rand = SystemRandom::new();
        let mut nonce_sequence = RandomNonceSequence { nonce: [0u8; NONCE_LEN], rand };

        let nonce = nonce_sequence.generate_nonce()?;
        nonce_sequence.nonce = nonce;

        Ok(nonce_sequence)
    }

    fn generate_nonce(&self) -> Result<[u8; NONCE_LEN], Unspecified> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        self.rand.fill(&mut nonce_bytes)?;

        Ok(nonce_bytes)
    }

    fn create(nonce: [u8; NONCE_LEN]) -> Self {
        let rand = SystemRandom::new();

        RandomNonceSequence { nonce, rand }
    }

    pub fn get_nonce(&self) -> [u8; NONCE_LEN] {
        self.nonce
    }
}

impl NonceSequence for RandomNonceSequence {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        let result = Nonce::try_assume_unique_for_key(&self.nonce)?;
        let new_nonce = self.generate_nonce()?;

        self.nonce = new_nonce;

        Ok(result)
    }
}

pub struct Aes256Gcm {
    key: Vec<u8>,
}

impl Aes256Gcm {
    pub fn new() -> Self {
        let key_string = env::var("AES_256_GCM_KEY").expect("AES_256_GCM_KEY must be available.");
        let key = hex::decode(key_string).expect("Expected to decode key from hex");

        Aes256Gcm { key }
    }

    pub fn encrypt(&self, mut data: Vec<u8>, aad: Vec<u8>) -> Result<EncryptResponse, Unspecified> {
        let nonce_sequence: RandomNonceSequence = RandomNonceSequence::new()?;
        let raw_nonce = nonce_sequence.get_nonce();
        let unbound_key = UnboundKey::new(&AES_256_GCM, &self.key)?;
        let mut sealing_key = SealingKey::new(unbound_key, nonce_sequence);
        let aad = Aad::from(aad);

        sealing_key.seal_in_place_append_tag(aad, &mut data)?;

        Ok(EncryptResponse { cypher: data, nonce: raw_nonce })
    }

    pub fn decrypt(&self, mut cypher: Vec<u8>, aad:Vec<u8>, nonce: [u8; NONCE_LEN]) -> Result<Vec<u8>, Unspecified> {
        let nonce_sequence: RandomNonceSequence = RandomNonceSequence::create(nonce);
        let unbound_key: UnboundKey = UnboundKey::new(&AES_256_GCM, &self.key)?;
        let aad = Aad::from(aad);
        let mut opening_key = OpeningKey::new(unbound_key, nonce_sequence);
        let decrypted_data = opening_key.open_in_place(aad, &mut cypher)?;

        Ok(decrypted_data.to_vec())
    }
}



