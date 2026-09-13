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
