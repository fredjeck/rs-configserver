use magic_crypt::{new_magic_crypt, MagicCryptTrait};

/// Encrypts a string using the provided key and encodes it to base64
pub fn encrypt_base64_string(key :&str, value:&str) -> String{ 
    let mc = new_magic_crypt!(key, 256);
    mc.encrypt_str_to_base64(value)
}

/// Decrypt a base64 encoded encrypted string using the provided key
pub fn decrypt_base64_string(key :&str, value:&str) -> Result<String, magic_crypt::MagicCryptError>{ 
    let mc = new_magic_crypt!(key, 256);
    mc.decrypt_base64_to_string(value)
}