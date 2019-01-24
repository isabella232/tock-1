pub trait CryptoKey {
    fn get_key_type(key_handle: *mut CryptoKey) -> u16;
    fn is_blank(key_handle: *mut CryptoKey) -> (u16, bool);
    fn mark_as_blank(key_handle: *mut CryptoKey) -> u16;
    //fn init_security_policy(policy: *mut DEVICE SPECIFIC POLICY??)
}

pub enum CryptoKeyStatus {
    Reserved = -32,
    Success = 0,
    Error = -1,
    UndefinedEncoding = -2,
}

pub enum CryptoKeyEncoding {
    Plaintext       = 0b00000010,
    BlankPlaintext  = 0x00000100,
    KeyStore        = 0x00001000,
    BlankKeyStore   = 0x00010000,
    KeyBlob         = 0x00100000,
    BlankKeyBlob    = 0x01000000,
}

#[repr(C)]
pub struct CryptoKeyPlaintext {
    key_material: *mut u8,
    key_length: u16,
}

#[repr(C)]
pub struct CryptoKeyKeyStore {
    // void* keyStore;
    key_length: u16,
    key_index: u16,
}

#[repr(C)]
pub struct CryptoKeyKeyBlob {
    key_blob: *mut u8,
    key_blob_length: u32,
}

union CryptoKeyType {
    crypto_plaintext: CryptoKeyPlaintext,
    crypto_key_store: CryptoKeyKeyStore,
    crypto_key_blob: CryptoKeyKeyBlob,
}

#[repr(C)]
pub struct CryptoKey {
    encoding: CryptoKeyEncoding,
    key_type: CryptoKeyType,
}

