use data_encoding::HEXLOWER_PERMISSIVE;
use ring::digest::{Context, SHA256, SHA256_OUTPUT_LEN};

pub type Hash = [u8; SHA256_OUTPUT_LEN];

pub fn compute_sha256(data: &[u8]) -> Hash {
    let mut context = Context::new(&SHA256);
    context.update(data);
    context.finish().as_ref().try_into().unwrap()
}

pub fn digest(data: &[u8]) -> String {
    HEXLOWER_PERMISSIVE.encode(data)
}

pub fn hex_to_bytes(hex: &str) -> Vec<u8> {
    HEXLOWER_PERMISSIVE.decode(hex.as_bytes()).unwrap()
}
