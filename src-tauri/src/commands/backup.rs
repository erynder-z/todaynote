// Backup commands: export and import the notes folder (notes plus embedded
// images) as a zip archive, with optional app-level encryption
// (XChaCha20-Poly1305 + Argon2id).
//
use crate::models::app_state::AppState;
use crate::models::response_types::ImportReport;
use chacha20poly1305::{
    aead::{AeadInOut, KeyInit, Tag},
    XChaCha20Poly1305, XNonce,
};
use rand::{rngs::SysRng, TryRng};
use std::fs;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;
use tauri::State;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// Magic bytes identifying an encrypted TodayNote backup (`.tnbk`).
const TNBK_MAGIC: &[u8; 8] = b"tnbk\x00\x01\x00\x00";
/// Argon2id salt length in bytes.
const SALT_LEN: usize = 16;
/// XChaCha20-Poly1305 nonce length in bytes.
const NONCE_LEN: usize = 24;
/// Poly1305 tag length in bytes.
const TAG_LEN: usize = 16;
/// Argon2id output key length (256 bits for XChaCha20-Poly1305).
const KEY_LEN: usize = 32;
/// Total container header: magic (8) + salt (16) + nonce (24).
const HEADER_LEN: usize = TNBK_MAGIC.len() + SALT_LEN + NONCE_LEN;
/// Argon2id memory cost (19 MiB) — the minimum configuration the OWASP
/// password-storage cheat sheet lists for Argon2id.
const ARGON2_M_COST: u32 = 19 * 1024;
/// Argon2id time cost (iterations).
const ARGON2_T_COST: u32 = 2;
/// Argon2id parallelism.
const ARGON2_P_COST: u32 = 1;

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

/// Encrypts `buf` in place (the Poly1305 tag is appended) and returns the
/// container header (magic + salt + nonce) to prepend.
fn encrypt_buffer_in_place(
    buf: &mut Vec<u8>,
    passphrase: &str,
) -> Result<[u8; HEADER_LEN], String> {
    let mut header = [0u8; HEADER_LEN];
    header[..TNBK_MAGIC.len()].copy_from_slice(TNBK_MAGIC);
    let mut rng = SysRng;
    rng.try_fill_bytes(&mut header[TNBK_MAGIC.len()..TNBK_MAGIC.len() + SALT_LEN])
        .map_err(|e| format!("Failed to generate salt: {e}"))?;
    rng.try_fill_bytes(&mut header[TNBK_MAGIC.len() + SALT_LEN..HEADER_LEN])
        .map_err(|e| format!("Failed to generate nonce: {e}"))?;

    let key = derive_key(
        passphrase,
        &header[TNBK_MAGIC.len()..TNBK_MAGIC.len() + SALT_LEN],
    )?;
    let cipher = XChaCha20Poly1305::new(&key.into());
    let nonce_arr: [u8; NONCE_LEN] = header[TNBK_MAGIC.len() + SALT_LEN..HEADER_LEN]
        .try_into()
        .map_err(|_| "Invalid nonce in backup header".to_string())?;
    let nonce: XNonce = nonce_arr.into();
    let tag = cipher
        .encrypt_inout_detached(&nonce, b"", buf.as_mut_slice().into())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    buf.extend_from_slice(&tag);
    Ok(header)
}

/// Decrypts a `.tnbk` container in place: on success `buf` (header +
/// ciphertext + tag) is truncated to just the plaintext.
fn decrypt_buffer_in_place(buf: &mut Vec<u8>, passphrase: &str) -> Result<(), String> {
    if buf.len() < HEADER_LEN {
        return Err("File is too small to be a valid encrypted backup".to_string());
    }
    if &buf[..TNBK_MAGIC.len()] != TNBK_MAGIC {
        return Err("File is not a valid encrypted backup".to_string());
    }
    if buf.len() < HEADER_LEN + TAG_LEN {
        return Err("File is too small to be a valid encrypted backup".to_string());
    }
    let key = derive_key(
        passphrase,
        &buf[TNBK_MAGIC.len()..TNBK_MAGIC.len() + SALT_LEN],
    )?;
    let cipher = XChaCha20Poly1305::new(&key.into());
    let nonce_arr: [u8; NONCE_LEN] = buf[TNBK_MAGIC.len() + SALT_LEN..HEADER_LEN]
        .try_into()
        .map_err(|_| "Invalid nonce in backup header".to_string())?;
    let nonce: XNonce = nonce_arr.into();

    let body_len = buf.len() - HEADER_LEN;
    let (ct, tag_bytes) = buf[HEADER_LEN..].split_at_mut(body_len - TAG_LEN);
    let tag_arr: [u8; TAG_LEN] = tag_bytes
        .try_into()
        .map_err(|_| "Invalid tag in backup".to_string())?;
    let tag: Tag<XChaCha20Poly1305> = tag_arr.into();
    cipher
        .decrypt_inout_detached(&nonce, b"", ct.into(), &tag)
        .map_err(|_| "Wrong password or corrupted backup".to_string())?;
    buf.truncate(HEADER_LEN + body_len - TAG_LEN);
    buf.drain(..HEADER_LEN);
    Ok(())
}

/// Encrypts the given plaintext bytes into the `.tnbk` container format:
/// `magic (8) | salt (16) | nonce (24) | ciphertext | tag`.
/// Test helper; production export streams to files via [`export_archive_to_path`].
#[cfg(test)]
fn encrypt_bytes(plaintext: &[u8], passphrase: &str) -> Result<Vec<u8>, String> {
    let mut buf = plaintext.to_vec();
    let header = encrypt_buffer_in_place(&mut buf, passphrase)?;
    let mut out = Vec::with_capacity(HEADER_LEN + buf.len());
    out.extend_from_slice(&header);
    out.extend_from_slice(&buf);
    Ok(out)
}

/// Decrypts `.tnbk` container bytes using the given passphrase.
/// Returns the recovered plaintext (the zip bytes) or an error on wrong password / corruption.
/// Test helper; production import uses [`decrypt_buffer_in_place`] on the file buffer.
#[cfg(test)]
fn decrypt_bytes(data: &[u8], passphrase: &str) -> Result<Vec<u8>, String> {
    let mut buf = data.to_vec();
    decrypt_buffer_in_place(&mut buf, passphrase)?;
    Ok(buf)
}

/// Streams notes and images into a zip archive written to `writer`, returning
/// the writer on success.
///
/// Each source file is copied into its archive entry with `io::copy`, so no
/// individual file is ever fully buffered in memory. Only files with an
/// allow-listed image extension are picked up from the `images` subfolder;
/// anything else there is skipped, mirroring how notes listing filters
/// non-`.md` files.
fn write_archive<W: Write + Seek>(
    writer: W,
    notes_folder: &Path,
    note_files: &[String],
) -> Result<W, String> {
    let mut zip = ZipWriter::new(writer);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for filename in note_files {
        let path = notes_folder.join(filename);
        let mut file = fs::File::open(&path)
            .map_err(|e| format!("Failed to read note '{}': {}", filename, e))?;
        let entry_name = format!("notes/{}", filename);
        zip.start_file(&entry_name, options)
            .map_err(|e| format!("Failed to add '{}' to archive: {}", filename, e))?;
        io::copy(&mut file, &mut zip)
            .map_err(|e| format!("Failed to write '{}' to archive: {}", filename, e))?;
    }

    let images_dir = notes_folder.join("images");
    if images_dir.is_dir() {
        let entries = fs::read_dir(&images_dir)
            .map_err(|e| format!("Failed to read images directory: {}", e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read images directory: {}", e))?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let filename = entry.file_name().to_string_lossy().into_owned();
            match path.extension().and_then(|e| e.to_str()) {
                Some(ext) if super::images::normalize_extension(ext).is_ok() => {}
                _ => continue,
            }
            let mut file = fs::File::open(&path)
                .map_err(|e| format!("Failed to read image '{}': {}", filename, e))?;
            let entry_name = format!("images/{}", filename);
            zip.start_file(&entry_name, options)
                .map_err(|e| format!("Failed to add '{}' to archive: {}", filename, e))?;
            io::copy(&mut file, &mut zip)
                .map_err(|e| format!("Failed to write '{}' to archive: {}", filename, e))?;
        }
    }

    zip.finish()
        .map_err(|e| format!("Failed to finalize archive: {}", e))
}

/// Builds the archive in memory. Only used by unit tests; production export
/// streams to a file via [`write_archive`].
#[cfg(test)]
fn build_archive_bytes(notes_folder: &Path, note_files: &[String]) -> Result<Vec<u8>, String> {
    let cursor = write_archive(Cursor::new(Vec::new()), notes_folder, note_files)?;
    Ok(cursor.into_inner())
}

/// Exports the notes folder to `dest` as a plain `.zip` or, when `password`
/// is given, an encrypted `.tnbk` container. Returns the number of bytes
/// written.
///
/// Plain exports stream the zip straight to the destination file. Encrypted
/// exports first stream the zip to a temp file, then encrypt that single
/// buffer in place and write header + ciphertext to `dest`.
fn export_archive_to_path(
    notes_folder: &Path,
    note_files: &[String],
    dest: &Path,
    password: Option<&str>,
) -> Result<usize, String> {
    match password.filter(|p| !p.is_empty()) {
        Some(pw) => {
            let tmp =
                std::env::temp_dir().join(format!("todaynote-export-{}.tmp", uuid::Uuid::new_v4()));
            let result: Result<usize, String> = (|| {
                let tmp_file = fs::File::create(&tmp)
                    .map_err(|e| format!("Failed to create temp archive: {}", e))?;
                write_archive(tmp_file, notes_folder, note_files)?;
                let mut file = fs::File::open(&tmp)
                    .map_err(|e| format!("Failed to read temp archive: {}", e))?;
                let size = file.metadata().map(|m| m.len() as usize).unwrap_or(0);
                // Reserve room for the tag up front so appending it cannot realloc.
                let mut buf = Vec::with_capacity(size + TAG_LEN);
                file.read_to_end(&mut buf)
                    .map_err(|e| format!("Failed to read temp archive: {}", e))?;
                let header = encrypt_buffer_in_place(&mut buf, pw)?;
                let mut out = fs::File::create(dest)
                    .map_err(|e| format!("Failed to write archive: {}", e))?;
                out.write_all(&header)
                    .and_then(|_| out.write_all(&buf))
                    .and_then(|_| out.flush())
                    .map_err(|e| format!("Failed to write archive: {}", e))?;
                Ok(HEADER_LEN + buf.len())
            })();
            let _ = fs::remove_file(&tmp);
            result
        }
        None => {
            let file =
                fs::File::create(dest).map_err(|e| format!("Failed to write archive: {}", e))?;
            write_archive(file, notes_folder, note_files)?;
            Ok(fs::metadata(dest).map(|m| m.len() as usize).unwrap_or(0))
        }
    }
}

/// Extracts notes and images from an opened archive into the notes folder.
///
/// `.md` entries land in the notes folder and image entries (validated against
/// the image extension allow-list) in its `images` subfolder. Entry names are
/// sanitized to their basename, so archives with or without `notes/`/`images/`
/// prefixes import the same way. When `overwrite` is false, existing files with
/// the same name are skipped. Each entry is streamed to disk with `io::copy`
/// instead of being buffered whole in memory.
fn import_archive_entries<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    notes_folder: &Path,
    overwrite: bool,
) -> ImportReport {
    let images_dir = notes_folder.join("images");
    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut errors: Vec<String> = Vec::new();
    let mut images_dir_ensured = false;

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

        // Route by extension: markdown goes to the notes folder, allow-listed
        // image types to the images directory, anything else is skipped.
        let is_image = !basename.ends_with(".md")
            && basename
                .rsplit_once('.')
                .map(|(_, ext)| super::images::normalize_extension(ext).is_ok())
                .unwrap_or(false);
        let dest = if is_image {
            if !images_dir_ensured {
                if let Err(e) = fs::create_dir_all(&images_dir) {
                    errors.push(format!("Failed to create images directory: {}", e));
                    continue;
                }
                images_dir_ensured = true;
            }
            images_dir.join(&basename)
        } else if basename.ends_with(".md") {
            notes_folder.join(&basename)
        } else {
            continue;
        };

        if dest.exists() && !overwrite {
            skipped += 1;
            continue;
        }

        let mut out = match fs::File::create(&dest) {
            Ok(f) => f,
            Err(e) => {
                errors.push(format!("Failed to write '{}': {}", basename, e));
                continue;
            }
        };
        if let Err(e) = io::copy(&mut entry, &mut out) {
            errors.push(format!("Failed to write '{}': {}", basename, e));
            continue;
        }
        imported += 1;
    }

    ImportReport {
        imported,
        skipped,
        errors,
    }
}

/// Imports a plain `.zip` or encrypted `.tnbk` archive from `src` into
/// `notes_folder` and returns an [`ImportReport`].
fn import_archive_from_path(
    src: &Path,
    notes_folder: &Path,
    password: Option<&str>,
    overwrite: bool,
) -> Result<ImportReport, String> {
    crate::services::note_manager::NoteManager::ensure_notes_folder_exists_in(notes_folder)?;

    let mut file = fs::File::open(src).map_err(|e| format!("Failed to read archive: {}", e))?;
    let mut prefix = [0u8; 8];
    let n = (&file)
        .read(&mut prefix)
        .map_err(|e| format!("Failed to read archive: {}", e))?;
    let magic_match = n >= TNBK_MAGIC.len() && &prefix[..TNBK_MAGIC.len()] == TNBK_MAGIC;

    match password.filter(|p| !p.is_empty()) {
        Some(pw) => {
            file.seek(SeekFrom::Start(0))
                .map_err(|e| format!("Failed to read archive: {}", e))?;
            let size = file.metadata().map(|m| m.len() as usize).unwrap_or(0);
            let mut buf = Vec::with_capacity(size);
            file.read_to_end(&mut buf)
                .map_err(|e| format!("Failed to read archive: {}", e))?;
            decrypt_buffer_in_place(&mut buf, pw)?;
            let mut archive = ZipArchive::new(Cursor::new(buf))
                .map_err(|e| format!("Failed to open archive: {}", e))?;
            Ok(import_archive_entries(
                &mut archive,
                notes_folder,
                overwrite,
            ))
        }
        None if magic_match => {
            Err("This backup is encrypted. Please provide the password.".to_string())
        }
        None => {
            file.seek(SeekFrom::Start(0))
                .map_err(|e| format!("Failed to read archive: {}", e))?;
            let mut archive =
                ZipArchive::new(file).map_err(|e| format!("Failed to open archive: {}", e))?;
            Ok(import_archive_entries(
                &mut archive,
                notes_folder,
                overwrite,
            ))
        }
    }
}

/// Exports all notes and embedded images in the configured folder to a zip
/// archive (optionally encrypted) written to a path chosen by the user via the
/// native save dialog.
///
/// When `password` is provided, the archive is encrypted into the `.tnbk`
/// container format before being written. Otherwise a plain `.zip` is written.
#[tauri::command]
pub async fn export_notes_archive(
    path: String,
    password: Option<String>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let note_manager = state.note_manager_snapshot()?;
    super::run_blocking(move || {
        let note_files = note_manager.get_sorted_note_files()?;
        export_archive_to_path(
            &note_manager.notes_folder,
            &note_files,
            Path::new(&path),
            password.as_deref(),
        )
    })
    .await
}

/// Imports notes and images from a backup archive at the given path.
///
/// Accepts both plain `.zip` files and encrypted `.tnbk` containers (when a `password`
/// is supplied). Notes are extracted into the current notes folder and images into
/// its `images` subfolder. When `overwrite` is false, existing files with the same
/// name are skipped. Only `.md` and allow-listed image entries are imported;
/// path-traversal entries are sanitized to their basename.
#[tauri::command]
pub async fn import_notes_archive(
    path: String,
    password: Option<String>,
    overwrite: bool,
    state: State<'_, AppState>,
) -> Result<ImportReport, String> {
    let notes_folder = state.note_manager()?.notes_folder.clone();

    // Reading the archive, decryption (Argon2id is CPU-heavy) and extracting
    // every entry all run on the blocking thread pool.
    super::run_blocking(move || {
        import_archive_from_path(
            Path::new(&path),
            &notes_folder,
            password.as_deref(),
            overwrite,
        )
    })
    .await
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
        let expected_min_size = HEADER_LEN + plaintext.len() + TAG_LEN;
        assert!(
            encrypted.len() >= expected_min_size,
            "Encrypted size should be at least header + plaintext + tag (actual: {}, expected: {})",
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

        assert!(
            total_len > HEADER_LEN,
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
        assert_eq!(
            ciphertext.len(),
            plaintext.len() + TAG_LEN,
            "Ciphertext should be plaintext plus tag"
        );
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
            encrypted.len() >= HEADER_LEN + TAG_LEN,
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
        assert_eq!(TAG_LEN, 16, "Poly1305 tag length should be 16 bytes");
        assert_eq!(
            KEY_LEN, 32,
            "Key length should be 32 bytes (256 bits) for XChaCha20-Poly1305"
        );
        assert_eq!(HEADER_LEN, 48, "Header should be 8 + 16 + 24 = 48 bytes");
        assert_eq!(
            ARGON2_M_COST,
            19 * 1024,
            "Argon2 memory cost should be 19 MiB (OWASP minimum)"
        );
        assert_eq!(
            ARGON2_T_COST, 2,
            "Argon2 time cost (iterations) should be 2"
        );
        assert_eq!(ARGON2_P_COST, 1, "Argon2 parallelism should be 1");
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
        assert_eq!(nonce_end, HEADER_LEN);

        let _magic: &[u8] = &encrypted[0..magic_end];
        let _salt: &[u8] = &encrypted[magic_end..salt_end];
        let _nonce: &[u8] = &encrypted[salt_end..nonce_end];
        let _ciphertext: &[u8] = &encrypted[nonce_end..];

        assert_eq!(_magic.len(), TNBK_MAGIC.len());
        assert_eq!(_salt.len(), SALT_LEN);
        assert_eq!(_nonce.len(), NONCE_LEN);
        assert!(!_ciphertext.is_empty());
    }

    // Archive Build / Import Tests

    use std::io::Cursor as IoCursor;
    use tempfile::tempdir;

    /// Creates a temp notes folder with two notes and an images subfolder
    /// containing two supported images and two unsupported files.
    fn setup_source_folder() -> (tempfile::TempDir, std::path::PathBuf, Vec<String>) {
        let src = tempdir().expect("Failed to create temp dir");
        let notes_folder = src.path().to_path_buf();
        fs::create_dir_all(notes_folder.join("images")).expect("Failed to create images dir");
        fs::write(notes_folder.join("2026-01-01.md"), "note a").expect("write failed");
        fs::write(notes_folder.join("2026-01-02.md"), "note b").expect("write failed");
        fs::write(notes_folder.join("images").join("abc.png"), b"png-bytes").expect("write failed");
        fs::write(notes_folder.join("images").join("def.jpg"), b"jpg-bytes").expect("write failed");
        fs::write(notes_folder.join("images").join("evil.svg"), b"<svg/>").expect("write failed");
        fs::write(notes_folder.join("images").join("meta.txt"), b"text").expect("write failed");

        let note_files = vec!["2026-01-02.md".to_string(), "2026-01-01.md".to_string()];
        (src, notes_folder, note_files)
    }

    #[test]
    fn test_build_archive_includes_notes_and_images_only() {
        let (_src, notes_folder, note_files) = setup_source_folder();

        let zip_bytes =
            build_archive_bytes(&notes_folder, &note_files).expect("Failed to build archive");

        let mut archive =
            ZipArchive::new(IoCursor::new(zip_bytes)).expect("Failed to open archive");
        let mut names: Vec<String> = (0..archive.len())
            .filter_map(|i| archive.by_index(i).ok().map(|e| e.name().to_string()))
            .collect();
        names.sort();
        assert_eq!(
            names,
            vec![
                "images/abc.png",
                "images/def.jpg",
                "notes/2026-01-01.md",
                "notes/2026-01-02.md",
            ]
        );
    }

    #[test]
    fn test_build_archive_without_images_dir() {
        let src = tempdir().expect("Failed to create temp dir");
        let notes_folder = src.path().to_path_buf();
        fs::write(notes_folder.join("2026-01-01.md"), "note a").expect("write failed");

        let zip_bytes = build_archive_bytes(&notes_folder, &["2026-01-01.md".to_string()])
            .expect("Failed to build archive");

        let archive = ZipArchive::new(IoCursor::new(zip_bytes)).expect("Failed to open archive");
        assert_eq!(archive.len(), 1);
    }

    #[test]
    fn test_import_roundtrip_restores_notes_and_images() {
        let (_src, notes_folder, note_files) = setup_source_folder();
        let zip_bytes =
            build_archive_bytes(&notes_folder, &note_files).expect("Failed to build archive");
        let mut archive =
            ZipArchive::new(IoCursor::new(zip_bytes)).expect("Failed to open archive");

        let dest = tempdir().expect("Failed to create temp dir");
        let report = import_archive_entries(&mut archive, dest.path(), false);

        assert_eq!(report.imported, 4, "Two notes and two images expected");
        assert_eq!(report.skipped, 0);
        assert!(report.errors.is_empty(), "Errors: {:?}", report.errors);
        assert_eq!(
            fs::read(dest.path().join("2026-01-01.md")).expect("note missing"),
            b"note a"
        );
        assert_eq!(
            fs::read(dest.path().join("images").join("abc.png")).expect("image missing"),
            b"png-bytes"
        );
        assert_eq!(
            fs::read(dest.path().join("images").join("def.jpg")).expect("image missing"),
            b"jpg-bytes"
        );
        assert!(!dest.path().join("images").join("evil.svg").exists());
        assert!(!dest.path().join("images").join("meta.txt").exists());
    }

    #[test]
    fn test_import_skips_existing_without_overwrite() {
        let (_src, notes_folder, note_files) = setup_source_folder();
        let zip_bytes =
            build_archive_bytes(&notes_folder, &note_files).expect("Failed to build archive");

        let dest = tempdir().expect("Failed to create temp dir");
        for overwrite in [false, true] {
            let mut archive =
                ZipArchive::new(IoCursor::new(zip_bytes.clone())).expect("Failed to open archive");
            let report = import_archive_entries(&mut archive, dest.path(), overwrite);
            assert_eq!(report.imported, 4, "overwrite={}", overwrite);
            assert_eq!(report.skipped, 0, "overwrite={}", overwrite);
        }

        let mut archive =
            ZipArchive::new(IoCursor::new(zip_bytes)).expect("Failed to open archive");
        let report = import_archive_entries(&mut archive, dest.path(), false);
        assert_eq!(report.imported, 0);
        assert_eq!(report.skipped, 4);
    }

    #[test]
    fn test_import_rejects_unsupported_image_entries() {
        let buf = IoCursor::new(Vec::new());
        let mut zip = ZipWriter::new(buf);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        zip.start_file("images/evil.svg", options)
            .expect("start failed");
        zip.write_all(b"<svg/>").expect("write failed");
        zip.start_file("images/ok.webp", options)
            .expect("start failed");
        zip.write_all(b"webp-bytes").expect("write failed");
        let buf = zip.finish().expect("finalize failed");

        let mut archive = ZipArchive::new(buf).expect("Failed to open archive");
        let dest = tempdir().expect("Failed to create temp dir");
        let report = import_archive_entries(&mut archive, dest.path(), false);

        assert_eq!(report.imported, 1, "Only the webp image should import");
        assert!(report.errors.is_empty());
        assert!(dest.path().join("images").join("ok.webp").exists());
        assert!(!dest.path().join("images").join("evil.svg").exists());
    }

    #[test]
    fn test_import_sanitizes_nested_entry_names() {
        let buf = IoCursor::new(Vec::new());
        let mut zip = ZipWriter::new(buf);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        zip.start_file("notes/2026-01-01.md", options)
            .expect("start failed");
        zip.write_all(b"note a").expect("write failed");
        let buf = zip.finish().expect("finalize failed");

        let mut archive = ZipArchive::new(buf).expect("Failed to open archive");
        let dest = tempdir().expect("Failed to create temp dir");
        let report = import_archive_entries(&mut archive, dest.path(), false);

        assert_eq!(report.imported, 1);
        assert!(dest.path().join("2026-01-01.md").exists());
        assert!(!dest.path().join("notes").exists());
    }

    // File-level export / import roundtrips (the streaming paths used in production)

    #[test]
    fn test_export_import_file_roundtrip_plain() {
        let (_src, notes_folder, note_files) = setup_source_folder();
        let out_dir = tempdir().expect("temp dir");
        let archive_path = out_dir.path().join("backup.zip");

        let written = export_archive_to_path(&notes_folder, &note_files, &archive_path, None)
            .expect("export failed");
        assert_eq!(
            written,
            fs::metadata(&archive_path).expect("archive missing").len() as usize
        );
        assert!(written > 0);
        // A plain zip must not look like an encrypted container.
        let raw = fs::read(&archive_path).expect("read archive");
        assert!(
            raw.len() < TNBK_MAGIC.len() || &raw[..TNBK_MAGIC.len()] != TNBK_MAGIC,
            "plain zip should not carry the tnbk magic"
        );

        let dest = tempdir().expect("temp dir");
        let report = import_archive_from_path(&archive_path, dest.path(), None, false)
            .expect("import failed");
        assert_eq!(report.imported, 4);
        assert!(report.errors.is_empty(), "Errors: {:?}", report.errors);
        assert_eq!(
            fs::read(dest.path().join("2026-01-01.md")).expect("note missing"),
            b"note a"
        );
        assert_eq!(
            fs::read(dest.path().join("images").join("abc.png")).expect("image missing"),
            b"png-bytes"
        );
        assert!(!dest.path().join("images").join("evil.svg").exists());
    }

    #[test]
    fn test_export_import_file_roundtrip_encrypted() {
        let (_src, notes_folder, note_files) = setup_source_folder();
        let out_dir = tempdir().expect("temp dir");
        let archive_path = out_dir.path().join("backup.tnbk");
        let password = "s3cret-pässword";

        let written =
            export_archive_to_path(&notes_folder, &note_files, &archive_path, Some(password))
                .expect("export failed");
        let raw = fs::read(&archive_path).expect("read archive");
        assert_eq!(raw.len(), written, "reported size should match file size");
        assert_eq!(&raw[..TNBK_MAGIC.len()], TNBK_MAGIC);

        // Encrypted archive without a password is rejected with a clear error.
        let dest = tempdir().expect("temp dir");
        let err = import_archive_from_path(&archive_path, dest.path(), None, false).unwrap_err();
        assert_eq!(
            err,
            "This backup is encrypted. Please provide the password."
        );

        // Wrong password fails with the usual message.
        let dest_wrong = tempdir().expect("temp dir");
        let err = import_archive_from_path(&archive_path, dest_wrong.path(), Some("wrong"), false)
            .unwrap_err();
        assert_eq!(err, "Wrong password or corrupted backup");

        let report = import_archive_from_path(&archive_path, dest.path(), Some(password), false)
            .expect("import failed");
        assert_eq!(report.imported, 4);
        assert!(report.errors.is_empty(), "Errors: {:?}", report.errors);
        assert_eq!(
            fs::read(dest.path().join("2026-01-02.md")).expect("note missing"),
            b"note b"
        );
        assert_eq!(
            fs::read(dest.path().join("images").join("def.jpg")).expect("image missing"),
            b"jpg-bytes"
        );
    }

    #[test]
    fn test_import_plain_zip_with_password_fails_cleanly() {
        let (_src, notes_folder, note_files) = setup_source_folder();
        let out_dir = tempdir().expect("temp dir");
        let archive_path = out_dir.path().join("backup.zip");
        export_archive_to_path(&notes_folder, &note_files, &archive_path, None).expect("export");

        let dest = tempdir().expect("temp dir");
        let err =
            import_archive_from_path(&archive_path, dest.path(), Some("pw"), false).unwrap_err();
        assert_eq!(err, "File is not a valid encrypted backup");
    }
}
