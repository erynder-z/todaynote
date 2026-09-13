// Backup commands: export and import the notes folder as a zip archive,
// with optional app-level encryption (XChaCha20-Poly1305 + Argon2id).
use crate::models::app_state::AppState;
use crate::models::response_types::ImportReport;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use rand::{rngs::SysRng, TryRng};
use std::fs;
use std::io::{Cursor, Read, Write};
use tauri::State;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// Magic bytes identifying an encrypted TodayNote backup (`.tnbk`).
const TNBK_MAGIC: &[u8; 8] = b"tnbk\x00\x01\x00\x00";
/// Argon2id salt length in bytes.
const SALT_LEN: usize = 16;
/// XChaCha20-Poly1305 nonce length in bytes.
const NONCE_LEN: usize = 24;
/// Argon2id output key length (256 bits for XChaCha20-Poly1305).
const KEY_LEN: usize = 32;
/// Argon2id memory cost (32 MiB).
const ARGON2_M_COST: u32 = 32 * 1024;
/// Argon2id time cost (iterations).
const ARGON2_T_COST: u32 = 3;
/// Argon2id parallelism.
const ARGON2_P_COST: u32 = 4;

/// Builds an in-memory zip archive containing every `.md` note in the configured folder.
///
/// Each note is stored under a `notes/` prefix with its bare filename, preserving the
/// flat `YYYY-MM-DD.md` layout. Returns the raw zip bytes.
fn build_zip_bytes(state: &State<'_, AppState>) -> Result<Vec<u8>, String> {
    let note_manager = state.note_manager()?;
    let notes_folder = note_manager.notes_folder.clone();
    let files = note_manager.get_sorted_note_files()?;

    let buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for filename in files {
        let path = notes_folder.join(&filename);
        let content =
            fs::read(&path).map_err(|e| format!("Failed to read note '{}': {}", filename, e))?;
        let entry_name = format!("notes/{}", filename);
        zip.start_file(&entry_name, options)
            .map_err(|e| format!("Failed to add '{}' to archive: {}", filename, e))?;
        zip.write_all(&content)
            .map_err(|e| format!("Failed to write '{}' to archive: {}", filename, e))?;
    }

    let buf = zip
        .finish()
        .map_err(|e| format!("Failed to finalize archive: {}", e))?;
    Ok(buf.into_inner())
}

/// Derives a 256-bit key from a passphrase and salt using Argon2id.
fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; KEY_LEN], String> {
    let params = argon2::Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(KEY_LEN))
        .map_err(|e| format!("Argon2 params error: {}", e))?;
    let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut key = [0u8; KEY_LEN];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    Ok(key)
}

/// Encrypts the given plaintext bytes into the `.tnbk` container format:
/// `magic (8) | salt (16) | nonce (24) | ciphertext`.
fn encrypt_bytes(plaintext: &[u8], passphrase: &str) -> Result<Vec<u8>, String> {
    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    let mut rng = SysRng;

    rng.try_fill_bytes(&mut salt)
        .map_err(|e| format!("Failed to generate salt: {e}"))?;

    rng.try_fill_bytes(&mut nonce_bytes)
        .map_err(|e| format!("Failed to generate nonce: {e}"))?;

    let key = derive_key(passphrase, &salt)?;
    let cipher = XChaCha20Poly1305::new(&key.into());
    let nonce: XNonce = nonce_bytes.into();
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut out = Vec::with_capacity(TNBK_MAGIC.len() + SALT_LEN + NONCE_LEN + ciphertext.len());
    out.extend_from_slice(TNBK_MAGIC);
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypts `.tnbk` container bytes using the given passphrase.
/// Returns the recovered plaintext (the zip bytes) or an error on wrong password / corruption.
fn decrypt_bytes(data: &[u8], passphrase: &str) -> Result<Vec<u8>, String> {
    let header = TNBK_MAGIC.len() + SALT_LEN + NONCE_LEN;
    if data.len() < header {
        return Err("File is too small to be a valid encrypted backup".to_string());
    }
    if &data[..TNBK_MAGIC.len()] != TNBK_MAGIC {
        return Err("File is not a valid encrypted backup".to_string());
    }
    let salt = &data[TNBK_MAGIC.len()..TNBK_MAGIC.len() + SALT_LEN];
    let nonce_bytes = &data[TNBK_MAGIC.len() + SALT_LEN..header];
    let ciphertext = &data[header..];

    let key = derive_key(passphrase, salt)?;
    let cipher = XChaCha20Poly1305::new(&key.into());
    let mut nonce_arr = [0u8; NONCE_LEN];
    nonce_arr.copy_from_slice(nonce_bytes);
    let nonce: XNonce = nonce_arr.into();
    cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|_| "Wrong password or corrupted backup".to_string())
}

/// Exports all notes in the configured folder to a zip archive (optionally encrypted)
/// written to a path chosen by the user via the native save dialog.
///
/// When `password` is provided, the zip bytes are encrypted into the `.tnbk` container
/// format before being written. Otherwise a plain `.zip` is written.
#[tauri::command]
pub async fn export_notes_archive(
    path: String,
    password: Option<String>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let zip_bytes = build_zip_bytes(&state)?;
    let bytes = match password {
        Some(ref pw) if !pw.is_empty() => encrypt_bytes(&zip_bytes, pw)?,
        _ => zip_bytes,
    };
    fs::write(&path, &bytes).map_err(|e| format!("Failed to write archive: {}", e))?;
    Ok(bytes.len())
}

/// Imports notes from a backup archive at the given path.
///
/// Accepts both plain `.zip` files and encrypted `.tnbk` containers (when a `password`
/// is supplied). Notes are extracted into the current notes folder. When `overwrite`
/// is false, existing notes with the same filename are skipped. Only `.md` entries are
/// imported; path-traversal entries are sanitized to their basename.
#[tauri::command]
pub async fn import_notes_archive(
    path: String,
    password: Option<String>,
    overwrite: bool,
    state: State<'_, AppState>,
) -> Result<ImportReport, String> {
    let raw = fs::read(&path).map_err(|e| format!("Failed to read archive: {}", e))?;

    let zip_bytes = match password {
        Some(ref pw) if !pw.is_empty() => decrypt_bytes(&raw, pw)?,
        _ => {
            // Auto-detect encrypted containers even without a password hint.
            if raw.len() >= TNBK_MAGIC.len() && &raw[..TNBK_MAGIC.len()] == TNBK_MAGIC {
                return Err("This backup is encrypted. Please provide the password.".to_string());
            }
            raw
        }
    };

    let note_manager = state.note_manager()?;
    note_manager.ensure_notes_folder_exists()?;
    let notes_folder = note_manager.notes_folder.clone();
    drop(note_manager);

    let mut archive = ZipArchive::new(Cursor::new(zip_bytes))
        .map_err(|e| format!("Failed to open archive: {}", e))?;

    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for i in 0..archive.len() {
        let mut entry = match archive.by_index(i) {
            Ok(e) => e,
            Err(e) => {
                errors.push(format!("Entry {}: could not read ({})", i, e));
                continue;
            }
        };

        let name = entry.name().to_string();
        if entry.is_dir() {
            continue;
        }

        // Sanitize to basename only, rejecting path traversal.
        let basename = match std::path::Path::new(&name).file_name() {
            Some(b) => b.to_string_lossy().into_owned(),
            None => continue,
        };
        if basename.is_empty() || basename.contains('/') || basename.contains('\\') {
            errors.push(format!("Skipped unsafe entry: {}", name));
            continue;
        }
        if !basename.ends_with(".md") {
            continue;
        }

        let dest = notes_folder.join(&basename);
        if dest.exists() && !overwrite {
            skipped += 1;
            continue;
        }

        let mut content = Vec::new();
        if let Err(e) = entry.read_to_end(&mut content) {
            errors.push(format!("Failed to read '{}': {}", basename, e));
            continue;
        }

        if let Err(e) = fs::write(&dest, &content) {
            errors.push(format!("Failed to write '{}': {}", basename, e));
            continue;
        }
        imported += 1;
    }

    Ok(ImportReport {
        imported,
        skipped,
        errors,
    })
}

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::response_types::ImportReport;

    // Encryption / Decryption Tests

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = b"Hello, World! This is a test of the encryption system.";
        let password = "my-secret-password";

        let encrypted = encrypt_bytes(plaintext, password).expect("Encryption failed");

        assert_eq!(
            &encrypted[..TNBK_MAGIC.len()],
            TNBK_MAGIC,
            "Magic bytes mismatch"
        );
        let expected_min_size = TNBK_MAGIC.len() + SALT_LEN + NONCE_LEN + plaintext.len();
        assert!(
            encrypted.len() >= expected_min_size,
            "Encrypted size should be at least header + plaintext (actual: {}, expected: {})",
            encrypted.len(),
            expected_min_size
        );

        let decrypted = decrypt_bytes(&encrypted, password).expect("Decryption failed");
        assert_eq!(
            decrypted, plaintext,
            "Decrypted content doesn't match plaintext"
        );
    }

    #[test]
    fn test_decrypt_with_wrong_password() {
        let plaintext = b"Secret data";
        let correct_password = "correct-password";
        let wrong_password = "wrong-password";

        let encrypted = encrypt_bytes(plaintext, correct_password).expect("Encryption failed");
        let result = decrypt_bytes(&encrypted, wrong_password);

        assert!(
            result.is_err(),
            "Decryption with wrong password should fail"
        );
        assert_eq!(
            result.unwrap_err(),
            "Wrong password or corrupted backup",
            "Error message mismatch"
        );
    }

    #[test]
    fn test_decrypt_invalid_magic_bytes() {
        let mut invalid_data = vec![0u8; 48];
        invalid_data[0..8].copy_from_slice(b"notatnbk");
        let password = "some-password";

        let result = decrypt_bytes(&invalid_data, password);
        assert!(result.is_err(), "Decryption with invalid magic should fail");
        assert_eq!(
            result.unwrap_err(),
            "File is not a valid encrypted backup",
            "Error message mismatch"
        );
    }

    #[test]
    fn test_decrypt_too_small() {
        let too_small = b"";
        let password = "some-password";

        let result = decrypt_bytes(too_small, password);
        assert!(
            result.is_err(),
            "Decryption with too small data should fail"
        );
        assert_eq!(
            result.unwrap_err(),
            "File is too small to be a valid encrypted backup",
            "Error message mismatch"
        );
    }

    #[test]
    fn test_encrypt_empty_plaintext() {
        let plaintext: &[u8] = &[];
        let password = "password";

        let encrypted = encrypt_bytes(plaintext, password).expect("Encryption failed");
        let decrypted = decrypt_bytes(&encrypted, password).expect("Decryption failed");

        assert_eq!(decrypted, plaintext, "Empty plaintext roundtrip failed");
    }

    #[test]
    fn test_encrypt_empty_password() {
        let plaintext = b"test data";
        let password = "";

        let encrypted =
            encrypt_bytes(plaintext, password).expect("Encryption with empty password should work");
        let decrypted = decrypt_bytes(&encrypted, password)
            .expect("Decryption with empty password should work");

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_large_data() {
        let plaintext: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        let password = "test-password";

        let encrypted =
            encrypt_bytes(&plaintext, password).expect("Encryption of large data failed");
        let decrypted =
            decrypt_bytes(&encrypted, password).expect("Decryption of large data failed");

        assert_eq!(decrypted, plaintext, "Large data roundtrip failed");
    }

    #[test]
    fn test_encrypt_with_special_characters() {
        let plaintext = b"\x00\x01\x02\xFF\xFE\xFD";
        let password = "password-with-special-chars: !@#$%^&*()";

        let encrypted = encrypt_bytes(plaintext, password).expect("Encryption failed");
        let decrypted = decrypt_bytes(&encrypted, password).expect("Decryption failed");

        assert_eq!(
            decrypted, plaintext,
            "Special character encryption/decryption failed"
        );
    }

    #[test]
    fn test_encrypt_unicode_content() {
        let plaintext = "Hello 世界 🌍 Ñoño".as_bytes();
        let password = "password";

        let encrypted = encrypt_bytes(plaintext, password).expect("Encryption failed");
        let decrypted = decrypt_bytes(&encrypted, password).expect("Decryption failed");

        assert_eq!(decrypted, plaintext, "Unicode content roundtrip failed");
    }

    // Key Derivation Tests

    #[test]
    fn test_key_derivation_deterministic() {
        let password = "test-password";
        let salt = [0u8; SALT_LEN];

        let key1 = derive_key(password, &salt).expect("Key derivation failed");
        let key2 = derive_key(password, &salt).expect("Key derivation failed");

        assert_eq!(key1, key2, "Same password and salt should produce same key");
    }

    #[test]
    fn test_key_derivation_different_salts() {
        let password = "test-password";
        let salt1 = [0u8; SALT_LEN];
        let salt2 = [1u8; SALT_LEN];

        let key1 = derive_key(password, &salt1).expect("Key derivation failed");
        let key2 = derive_key(password, &salt2).expect("Key derivation failed");

        assert_ne!(key1, key2, "Different salts should produce different keys");
    }

    #[test]
    fn test_key_derivation_different_passwords() {
        let salt = [0u8; SALT_LEN];
        let password1 = "password1";
        let password2 = "password2";

        let key1 = derive_key(password1, &salt).expect("Key derivation failed");
        let key2 = derive_key(password2, &salt).expect("Key derivation failed");

        assert_ne!(
            key1, key2,
            "Different passwords should produce different keys"
        );
    }

    #[test]
    fn test_key_derivation_correct_length() {
        let password = "test-password";
        let salt = [0u8; SALT_LEN];

        let key = derive_key(password, &salt).expect("Key derivation failed");
        assert_eq!(
            key.len(),
            KEY_LEN,
            "Key should be correct length (32 bytes for XChaCha20-Poly1305)"
        );
    }

    // Encrypted Container Format Tests

    #[test]
    fn test_tnbk_container_structure() {
        let plaintext = b"test data";
        let password = "password";

        let encrypted = encrypt_bytes(plaintext, password).expect("Encryption failed");

        let total_len = encrypted.len();
        let header_len = TNBK_MAGIC.len() + SALT_LEN + NONCE_LEN;

        assert!(
            total_len > header_len,
            "Encrypted data should be larger than header"
        );

        assert_eq!(
            &encrypted[0..TNBK_MAGIC.len()],
            TNBK_MAGIC,
            "Magic bytes at start"
        );

        let salt_start = TNBK_MAGIC.len();
        let salt_end = salt_start + SALT_LEN;
        assert_eq!(
            encrypted[salt_start..salt_end].len(),
            SALT_LEN,
            "Salt should be correct length"
        );

        let nonce_start = salt_end;
        let nonce_end = nonce_start + NONCE_LEN;
        assert_eq!(
            encrypted[nonce_start..nonce_end].len(),
            NONCE_LEN,
            "Nonce should be correct length"
        );

        let ciphertext = &encrypted[nonce_end..];
        assert!(!ciphertext.is_empty(), "Ciphertext should not be empty");
    }

    #[test]
    fn test_container_magic_bytes() {
        assert_eq!(TNBK_MAGIC.len(), 8, "Magic bytes should be 8 bytes");
        assert_eq!(
            TNBK_MAGIC, b"tnbk\x00\x01\x00\x00",
            "Magic bytes should be 'tnbk\\x00\\x01\\x00\\x00'"
        );
    }

    #[test]
    fn test_auto_detect_encrypted_container() {
        let plaintext = b"test";
        let password = "password";
        let encrypted = encrypt_bytes(plaintext, password).expect("Encryption failed");

        assert_eq!(
            &encrypted[..TNBK_MAGIC.len()],
            TNBK_MAGIC,
            "Encrypted container should have magic bytes"
        );

        assert!(
            encrypted.len() >= TNBK_MAGIC.len() + SALT_LEN + NONCE_LEN,
            "Encrypted container should have complete header"
        );
    }

    // Constants Tests

    #[test]
    fn test_constants_values() {
        assert_eq!(
            TNBK_MAGIC, b"tnbk\x00\x01\x00\x00",
            "Magic bytes should identify tnbk format"
        );
        assert_eq!(SALT_LEN, 16, "Salt length should be 16 bytes for Argon2id");
        assert_eq!(
            NONCE_LEN, 24,
            "Nonce length should be 24 bytes for XChaCha20-Poly1305"
        );
        assert_eq!(
            KEY_LEN, 32,
            "Key length should be 32 bytes (256 bits) for XChaCha20-Poly1305"
        );
        assert_eq!(
            ARGON2_M_COST,
            32 * 1024,
            "Argon2 memory cost should be 32 MiB"
        );
        assert_eq!(
            ARGON2_T_COST, 3,
            "Argon2 time cost (iterations) should be 3"
        );
        assert_eq!(ARGON2_P_COST, 4, "Argon2 parallelism should be 4");
    }

    // ImportReport Tests

    #[test]
    fn test_import_report_structure() {
        let report = ImportReport {
            imported: 5,
            skipped: 2,
            errors: vec!["Error 1".to_string(), "Error 2".to_string()],
        };

        let json = serde_json::to_string(&report).expect("Failed to serialize report");
        let deserialized: ImportReport =
            serde_json::from_str(&json).expect("Failed to deserialize report");

        assert_eq!(
            deserialized.imported, report.imported,
            "Imported count should match"
        );
        assert_eq!(
            deserialized.skipped, report.skipped,
            "Skipped count should match"
        );
        assert_eq!(deserialized.errors, report.errors, "Errors should match");
    }

    #[test]
    fn test_import_report_empty() {
        let report = ImportReport {
            imported: 0,
            skipped: 0,
            errors: vec![],
        };

        assert!(report.errors.is_empty(), "Errors should be empty");
        assert_eq!(report.imported, 0, "Imported count should be 0");
        assert_eq!(report.skipped, 0, "Skipped count should be 0");
    }

    #[test]
    fn test_import_report_with_errors() {
        let errors = vec![
            "Failed to read file: permission denied".to_string(),
            "Invalid filename: contains path traversal".to_string(),
            "Unsupported file type: .txt".to_string(),
        ];

        let report = ImportReport {
            imported: 10,
            skipped: 5,
            errors,
        };

        assert_eq!(report.errors.len(), 3, "Should have 3 errors");
        assert!(
            report.errors[0].contains("permission denied"),
            "First error should contain 'permission denied'"
        );
        assert!(
            report.errors[1].contains("path traversal"),
            "Second error should contain 'path traversal'"
        );
        assert!(
            report.errors[2].contains(".txt"),
            "Third error should contain '.txt'"
        );
    }

    // Edge Cases and Error Handling Tests

    #[test]
    fn test_decrypt_with_empty_data() {
        let empty_data: &[u8] = &[];
        let password = "password";

        let result = decrypt_bytes(empty_data, password);
        assert!(result.is_err(), "Decrypting empty data should fail");
    }

    #[test]
    fn test_decrypt_with_incomplete_header() {
        let incomplete_header = b"tnbk\x00\x01\x00\x00";
        let password = "password";

        let result = decrypt_bytes(incomplete_header, password);
        assert!(
            result.is_err(),
            "Decrypting with incomplete header should fail"
        );
    }

    #[test]
    fn test_encrypt_decrypt_with_long_password() {
        let plaintext = b"test data";
        let long_password = "a".repeat(1000);

        let encrypted = encrypt_bytes(plaintext, &long_password)
            .expect("Encryption with long password should work");
        let decrypted = decrypt_bytes(&encrypted, &long_password)
            .expect("Decryption with long password should work");

        assert_eq!(
            decrypted, plaintext,
            "Long password encryption/decryption should work"
        );
    }

    #[test]
    fn test_encrypt_decrypt_with_unicode_password() {
        let plaintext = b"test data";
        let unicode_password = "密码123 🔑 пароль";

        let encrypted = encrypt_bytes(plaintext, unicode_password)
            .expect("Encryption with unicode password should work");
        let decrypted = decrypt_bytes(&encrypted, unicode_password)
            .expect("Decryption with unicode password should work");

        assert_eq!(
            decrypted, plaintext,
            "Unicode password encryption/decryption should work"
        );
    }

    #[test]
    fn test_derive_key_with_empty_password() {
        let empty_password = "";
        let salt = [0u8; SALT_LEN];

        let result = derive_key(empty_password, &salt);
        assert!(
            result.is_ok(),
            "Key derivation with empty password should succeed"
        );
        let key = result.unwrap();
        assert_eq!(key.len(), KEY_LEN, "Key should still be correct length");
    }

    #[test]
    fn test_multiple_encryption_operations_produce_different_ciphertexts() {
        let plaintext = b"same plaintext";
        let password = "same password";

        let encrypted1 = encrypt_bytes(plaintext, password).expect("First encryption failed");
        let encrypted2 = encrypt_bytes(plaintext, password).expect("Second encryption failed");

        assert_ne!(
            encrypted1, encrypted2,
            "Each encryption should produce different ciphertext (different salt/nonce)"
        );

        let decrypted1 = decrypt_bytes(&encrypted1, password).expect("First decryption failed");
        let decrypted2 = decrypt_bytes(&encrypted2, password).expect("Second decryption failed");

        assert_eq!(
            decrypted1, plaintext,
            "First decryption should match plaintext"
        );
        assert_eq!(
            decrypted2, plaintext,
            "Second decryption should match plaintext"
        );
    }

    // Container Format Validation Tests

    #[test]
    fn test_container_header_boundaries() {
        let plaintext = b"test";
        let password = "password";

        let encrypted = encrypt_bytes(plaintext, password).expect("Encryption failed");

        let magic_end = TNBK_MAGIC.len();
        let salt_end = magic_end + SALT_LEN;
        let nonce_end = salt_end + NONCE_LEN;

        assert_eq!(magic_end, 8, "Magic bytes end at position 8");
        assert_eq!(salt_end, 24, "Salt ends at position 24 (8 + 16)");
        assert_eq!(nonce_end, 48, "Nonce ends at position 48 (24 + 24)");

        let _magic: &[u8] = &encrypted[0..magic_end];
        let _salt: &[u8] = &encrypted[magic_end..salt_end];
        let _nonce: &[u8] = &encrypted[salt_end..nonce_end];
        let _ciphertext: &[u8] = &encrypted[nonce_end..];

        assert_eq!(_magic.len(), TNBK_MAGIC.len());
        assert_eq!(_salt.len(), SALT_LEN);
        assert_eq!(_nonce.len(), NONCE_LEN);
        assert!(_ciphertext.len() > 0);
    }
}
