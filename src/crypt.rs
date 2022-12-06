use magic_crypt::{new_magic_crypt, MagicCryptTrait};


pub fn encrypt_str(key :&str, value:&str) -> String{ 
    let mc = new_magic_crypt!(key, 256);
    mc.encrypt_str_to_base64(value)
}