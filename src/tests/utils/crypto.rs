use openssl::pkey::PKey;
use openssl::rsa::Rsa;

use crate::services::jsonwebtoken::KeyPair;

pub fn generate_keypair() -> KeyPair {
    let rsa = Rsa::generate(2048).unwrap();
    let pkey = PKey::from_rsa(rsa).unwrap();
    let private_key = pkey.private_key_to_pem_pkcs8().unwrap();
    let public_key = pkey.public_key_to_pem().unwrap();

    KeyPair::from_rsa_pem(private_key, public_key).unwrap()
}
