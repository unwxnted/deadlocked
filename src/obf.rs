use std::sync::LazyLock;

static OBF_KEY: LazyLock<u8> = LazyLock::new(compute_key);

fn compute_key() -> u8 {
    let hash = env!("GIT_HASH");
    let mut key = hash.as_bytes().iter().fold(0u8, |acc, &b| acc ^ b);
    if key == 0 {
        key = 0xA7;
    }
    key
}

pub fn key() -> u8 {
    *OBF_KEY
}

pub const fn xor_encrypt<const N: usize>(data: &[u8], key: u8) -> [u8; N] {
    let mut result = [0u8; N];
    let mut i = 0;
    while i < N {
        result[i] = data[i] ^ key;
        i += 1;
    }
    result
}

pub struct Obfuscated<const N: usize> {
    pub encrypted: [u8; N],
}

impl<const N: usize> Obfuscated<N> {
    pub const fn new(encrypted: [u8; N]) -> Self {
        Self { encrypted }
    }

    pub fn decrypt(&self) -> String {
        let key = key();
        let mut result = Vec::with_capacity(N);
        for &b in &self.encrypted {
            result.push(b ^ key);
        }
        unsafe { String::from_utf8_unchecked(result) }
    }
}

#[macro_export]
macro_rules! obfstr {
    ($s:literal) => {{
        const KEY: u8 = $crate::obf::__compile_time_key();
        const BYTES: &[u8] = $s.as_bytes();
        const LEN: usize = BYTES.len();
        const ENCRYPTED: [u8; LEN] = $crate::obf::xor_encrypt::<LEN>(BYTES, KEY);
        $crate::obf::Obfuscated::new(ENCRYPTED)
    }};
}

/// Must be called inside obfstr! macro expansion.
/// Uses env!("GIT_HASH") at the call site to derive a compile-time encryption key.
#[doc(hidden)]
pub const fn __compile_time_key() -> u8 {
    let hash = env!("GIT_HASH");
    let bytes = hash.as_bytes();
    let mut i = 0;
    let mut key = 0u8;
    while i < bytes.len() {
        key ^= bytes[i];
        i += 1;
    }
    if key == 0 { 0xA7 } else { key }
}
