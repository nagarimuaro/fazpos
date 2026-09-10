/// Konversi string legacy ke tipe f64 (menangani titik/koma dan spasi)
pub fn konversi_f64(raw: &str) -> f64 {
    let bersih = raw.trim().replace(',', ".").replace(' ', "");
    bersih.parse::<f64>().unwrap_or(0.0)
}

/// Konversi string legacy ke i64
pub fn konversi_i64(raw: &str) -> i64 {
    let bersih = raw.trim().replace(' ', "");
    // Menangani float string seperti "10.00" -> 10
    if let Ok(f) = bersih.parse::<f64>() {
        return f as i64;
    }
    bersih.parse::<i64>().unwrap_or(0)
}

/// Konversi string legacy ke bool ('true', 'false', '1', '0', 'yes', 'no')
pub fn konversi_bool(raw: &str) -> bool {
    let lower = raw.trim().to_lowercase();
    matches!(lower.as_str(), "true" | "1" | "yes" | "y" | "t")
}

/// Bersihkan string dari quote SQL dan escape character
pub fn unquote_sql(raw: &str) -> String {
    let s = raw.trim();
    let unquoted = if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
        if s.len() >= 2 {
            &s[1..s.len() - 1]
        } else {
            ""
        }
    } else {
        s
    };

    unquoted.replace(r"\'", "'").replace("''", "'").replace(r#"\""#, "\"")
}
