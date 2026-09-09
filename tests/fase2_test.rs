use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use fazpos::license::machine_id::MachineId;
use fazpos::license::payload::{LicensePayload, SignedLicenseToken};
use fazpos::license::verification::{LicenseStatus, LicenseVerifier};
use std::fs;

fn buat_test_keypair() -> (SigningKey, [u8; 32]) {
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed).unwrap();
    let signing_key = SigningKey::from_bytes(&seed);
    let pub_bytes = signing_key.verifying_key().to_bytes();
    (signing_key, pub_bytes)
}

#[test]
fn test_machine_id_format_dan_deterministik() {
    let raw = "GUID-12345|INTEL-I7|DESKTOP-POS-01";
    let id1 = MachineId::format_hash(raw);
    let id2 = MachineId::format_hash(raw);

    assert_eq!(id1, id2, "Machine ID harus deterministik untuk input hardware yang sama");
    assert!(id1.starts_with("RTG-"), "Machine ID harus berawalan prefix RTG-");
    assert_eq!(id1.len(), 23, "Format RTG-XXXX-XXXX-XXXX-XXXX panjangnya 23 karakter");

    // Test Machine ID aktual dari mesin ini
    let current_id = MachineId::dapatkan();
    assert!(current_id.starts_with("RTG-"));
    assert_eq!(current_id.len(), 23);
}

#[test]
fn test_lisensi_valid_ed25519() {
    let (signing_key, pub_bytes) = buat_test_keypair();

    let machine_id = "RTG-1A2B-3C4D-5E6F-7890";
    let payload = LicensePayload::baru_lifetime(machine_id, "Toko Sentosa Jaya", "2026-09-09");
    let payload_json = serde_json::to_string(&payload).unwrap();

    let signature = signing_key.sign(payload_json.as_bytes());
    let signed_token = SignedLicenseToken {
        payload_json,
        signature_b64: B64.encode(signature.to_bytes()),
    };

    let token_str = signed_token.ke_string_token().unwrap();

    // Verifikasi dengan public key dan machine_id yang cocok
    let result = LicenseVerifier::verifikasi_dengan_public_key(&token_str, machine_id, &pub_bytes);
    assert!(result.is_ok());
    let verified = result.unwrap();
    assert_eq!(verified.nama_toko, "Toko Sentosa Jaya");
    assert_eq!(verified.machine_id, machine_id);
    assert_eq!(verified.tipe, "LIFETIME");
}

#[test]
fn test_lisensi_gagal_jika_machine_id_berbeda() {
    let (signing_key, pub_bytes) = buat_test_keypair();

    let machine_id_asli = "RTG-1111-2222-3333-4444";
    let machine_id_bajakan = "RTG-9999-8888-7777-6666";

    let payload = LicensePayload::baru_lifetime(machine_id_asli, "Toko Sentosa", "2026-09-09");
    let payload_json = serde_json::to_string(&payload).unwrap();

    let signature = signing_key.sign(payload_json.as_bytes());
    let signed_token = SignedLicenseToken {
        payload_json,
        signature_b64: B64.encode(signature.to_bytes()),
    };

    let token_str = signed_token.ke_string_token().unwrap();

    // Jalankan di mesin berbeda -> harus ditolak
    let result = LicenseVerifier::verifikasi_dengan_public_key(&token_str, machine_id_bajakan, &pub_bytes);
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("Lisensi terikat pada mesin"));
}

#[test]
fn test_lisensi_gagal_jika_payload_dimodifikasi() {
    let (signing_key, pub_bytes) = buat_test_keypair();

    let machine_id = "RTG-1A2B-3C4D-5E6F-7890";
    let payload = LicensePayload::baru_lifetime(machine_id, "Toko Asli", "2026-09-09");
    let payload_json = serde_json::to_string(&payload).unwrap();

    let signature = signing_key.sign(payload_json.as_bytes());

    // Hacker mencoba ubah nama toko di payload json tanpa private key
    let tampered_json = payload_json.replace("Toko Asli", "Toko Bajakan");

    let tampered_token = SignedLicenseToken {
        payload_json: tampered_json,
        signature_b64: B64.encode(signature.to_bytes()),
    };

    let token_str = tampered_token.ke_string_token().unwrap();

    // Verifikasi harus gagal karena signature tidak cocok dengan isi payload yang diubah
    let result = LicenseVerifier::verifikasi_dengan_public_key(&token_str, machine_id, &pub_bytes);
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("Tanda tangan digital lisensi PALSU"));
}

#[test]
fn test_penyimpanan_file_lisensi_lokal_dan_anti_tamper() {
    let (signing_key, pub_bytes) = buat_test_keypair();

    let machine_id = "RTG-TEST-FILE-0001-0002";
    let payload = LicensePayload::baru_lifetime(machine_id, "Toko Sukses", "2026-09-09");
    let payload_json = serde_json::to_string(&payload).unwrap();
    let signature = signing_key.sign(payload_json.as_bytes());

    let signed_token = SignedLicenseToken {
        payload_json,
        signature_b64: B64.encode(signature.to_bytes()),
    };
    let token_str = signed_token.ke_string_token().unwrap();

    let temp_file = std::env::temp_dir().join("test_pos_license.lic");

    // 1. Simpan lisensi
    LicenseVerifier::simpan_ke_file(&temp_file, &token_str, machine_id).unwrap();

    // 2. Baca kembali di mesin yang sama -> harus Aktif
    let status = LicenseVerifier::baca_dari_file_dengan_key(&temp_file, machine_id, &pub_bytes);
    assert!(matches!(status, LicenseStatus::Aktif(_)));

    // 3. Coba baca file yang sama dari mesin lain -> harus Ditolak (anti-tamper checksum mismatch)
    let status_mesin_lain = LicenseVerifier::baca_dari_file_dengan_key(
        &temp_file,
        "RTG-MESIN-LAIN-0000-0000",
        &pub_bytes,
    );
    assert!(matches!(status_mesin_lain, LicenseStatus::TidakValid(_)));

    // 4. Rusak isi file lisensi -> harus Ditolak
    fs::write(&temp_file, "KONTEN_RUSAK_TIDAK_VALID\nCHECKSUM_SALAH").unwrap();
    let status_rusak = LicenseVerifier::baca_dari_file_dengan_key(&temp_file, machine_id, &pub_bytes);
    assert!(matches!(status_rusak, LicenseStatus::TidakValid(_)));

    // Cleanup
    let _ = fs::remove_file(temp_file);
}
