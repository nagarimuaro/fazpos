use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use fazpos::license::payload::{LicensePayload, SignedLicenseToken};
use fazpos::license::verification::LicenseVerifier;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "keygen" => {
            jalankan_keygen();
        }
        "sign" => {
            if args.len() < 4 {
                eprintln!("Error: Perintah sign butuh <machine_id> dan <nama_toko>");
                eprintln!("Contoh: cargo run --bin license_cli sign RTG-XXXX-XXXX-XXXX-XXXX \"Toko Barokah\"");
                return;
            }
            let machine_id = &args[2];
            let nama_toko = &args[3];
            let priv_path = args.get(4).map(|s| s.as_str()).unwrap_or("vendor_private.key");
            jalankan_sign(machine_id, nama_toko, priv_path);
        }
        "verify" => {
            if args.len() < 4 {
                eprintln!("Error: Perintah verify butuh <token> dan <machine_id>");
                return;
            }
            let token = &args[2];
            let machine_id = &args[3];
            jalankan_verify(token, machine_id);
        }
        _ => {
            print_usage();
        }
    }
}

fn print_usage() {
    println!("=== Tool Internal Vendor Lisensi FAZPOS ===");
    println!("Perintah yang tersedia:");
    println!("  1. keygen                                   : Buat keypair Ed25519 baru");
    println!("  2. sign <machine_id> <nama_toko> [key_file] : Buat token lisensi untuk pelanggan");
    println!("  3. verify <token> <machine_id>              : Verifikasi token lisensi");
}

fn jalankan_keygen() {
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed).expect("Gagal membaca OS entropy");
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();

    let priv_bytes = signing_key.to_bytes();
    let pub_bytes = verifying_key.to_bytes();

    fs::write("vendor_private.key", priv_bytes).expect("Gagal menyimpan vendor_private.key");
    fs::write("vendor_public.key", pub_bytes).expect("Gagal menyimpan vendor_public.key");

    println!("Keypair Ed25519 berhasil dibuat!");
    println!("  - File private: vendor_private.key (RAHASIA, JANGAN DISTRIBUSIKAN)");
    println!("  - File public : vendor_public.key");
    println!("\nPublic key hex untuk dimasukkan ke EMBEDDED_VENDOR_PUBLIC_KEY:");
    print!("pub const EMBEDDED_VENDOR_PUBLIC_KEY: [u8; 32] = [");
    for (i, b) in pub_bytes.iter().enumerate() {
        if i % 8 == 0 {
            print!("\n    ");
        }
        print!("0x{:02x}, ", b);
    }
    println!("\n];");
}

fn jalankan_sign(machine_id: &str, nama_toko: &str, priv_key_path: &str) {
    if !Path::new(priv_key_path).exists() {
        eprintln!("Error: File private key [{}] tidak ditemukan!", priv_key_path);
        eprintln!("Jalankan perintah 'keygen' terlebih dahulu.");
        return;
    }

    let priv_bytes = fs::read(priv_key_path).expect("Gagal membaca private key");
    if priv_bytes.len() != 32 {
        eprintln!("Error: Ukuran private key tidak valid (harus 32 bytes)");
        return;
    }

    let mut key_arr = [0u8; 32];
    key_arr.copy_from_slice(&priv_bytes);
    let signing_key = SigningKey::from_bytes(&key_arr);

    let tanggal_sekarang = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let payload = LicensePayload::baru_lifetime(machine_id, nama_toko, tanggal_sekarang);
    let payload_json = serde_json::to_string(&payload).expect("Gagal serialisasi payload");

    // Sign payload menggunakan Ed25519
    let signature = signing_key.sign(payload_json.as_bytes());
    let signature_b64 = B64.encode(signature.to_bytes());

    let signed_token = SignedLicenseToken {
        payload_json,
        signature_b64,
    };

    let token_string = signed_token.ke_string_token().expect("Gagal encode token");

    println!("=== TOKEN AKTIVASI BERHASIL DIGENERATE ===");
    println!("Toko       : {}", nama_toko);
    println!("Machine ID : {}", machine_id);
    println!("Tipe       : LIFETIME");
    println!("\nSalin token berikut dan kirimkan ke pelanggan:");
    println!("--------------------------------------------------");
    println!("{}", token_string);
    println!("--------------------------------------------------");
}

fn jalankan_verify(token: &str, machine_id: &str) {
    match LicenseVerifier::verifikasi(token, machine_id) {
        Ok(payload) => {
            println!("VALID: Token lisensi sah!");
            println!("  Toko       : {}", payload.nama_toko);
            println!("  Machine ID : {}", payload.machine_id);
            println!("  Tipe       : {}", payload.tipe);
            println!("  Terbit     : {}", payload.tanggal_terbit);
        }
        Err(err) => {
            eprintln!("TIDAK VALID: {}", err);
        }
    }
}
