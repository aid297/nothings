use crate::digests::bcrypt::Bcrypt;
use crate::digests::md5::Md5Hash;
use crate::digests::sha::Sha256Hash;

#[test]
fn test_hash() {
    let plaintext = "origin_password".to_string();
    match Bcrypt::new(&plaintext).hash() {
        Ok(hash) => {
            println!("{}", hash);
        }
        Err(e) => panic!("{}", e),
    }
}

#[test]
fn test_check() {
    let plaintext = "origin_password".to_string();
    let hashed = "$2b$12$NQYkakzqy7H53lelLjI5Kemzd8P8lQ5FoM/bSIxtVtiHAYOKD9d/O".to_string();
    match Bcrypt::new(&plaintext).check(&hashed) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => panic!("{}", e),
    }
}

#[test]
fn test_md5_encode() {
    let plaintext = "origin_password".to_string();
    println!("{}", Md5Hash::new(&plaintext.as_str()).hash())
}

#[test]
fn test_sha256_encode(){
    let plaintext = "origin_password".to_string();
    println!("{}", Sha256Hash::new(&plaintext.as_str()).hash())
}
