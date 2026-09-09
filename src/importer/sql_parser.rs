use crate::importer::types::unquote_sql;

#[derive(Debug, Clone, PartialEq)]
pub struct RawInsertStatement {
    pub tabel: String,
    pub kolom: Option<Vec<String>>,
    pub rows: Vec<Vec<String>>,
}

pub struct SqlDumpParser;

impl SqlDumpParser {
    /// Parse seluruh isi teks file SQL dump dan ekstrak seluruh statement INSERT
    pub fn parse_dump(sql_content: &str) -> Vec<RawInsertStatement> {
        let mut results = Vec::new();
        let mut pos = 0;
        let bytes = sql_content.as_bytes();
        let len = bytes.len();

        while pos < len {
            // Lewati komentar SQL (-- atau /* ... */)
            if pos + 1 < len && bytes[pos] == b'-' && bytes[pos + 1] == b'-' {
                while pos < len && bytes[pos] != b'\n' {
                    pos += 1;
                }
                continue;
            }
            if pos + 1 < len && bytes[pos] == b'/' && bytes[pos + 1] == b'*' {
                pos += 2;
                while pos + 1 < len && !(bytes[pos] == b'*' && bytes[pos + 1] == b'/') {
                    pos += 1;
                }
                pos += 2;
                continue;
            }

            // Cari keyword INSERT INTO
            if Self::cek_keyword(sql_content, pos, "INSERT INTO") {
                pos += 11; // panjang "INSERT INTO"
                if let Some((stmt, new_pos)) = Self::ekstrak_insert(sql_content, pos) {
                    results.push(stmt);
                    pos = new_pos;
                    continue;
                }
            }

            pos += 1;
        }

        results
    }

    fn cek_keyword(text: &str, pos: usize, keyword: &str) -> bool {
        let kw_len = keyword.len();
        if pos + kw_len <= text.len() {
            text[pos..pos + kw_len].eq_ignore_ascii_case(keyword)
        } else {
            false
        }
    }

    /// Ekstrak nama tabel, daftar kolom (opsional), dan baris-baris VALUES (...)
    fn ekstrak_insert(text: &str, mut pos: usize) -> Option<(RawInsertStatement, usize)> {
        let bytes = text.as_bytes();
        let len = bytes.len();

        // 1. Lewati spasi sebelum nama tabel
        while pos < len && (bytes[pos] as char).is_whitespace() {
            pos += 1;
        }

        // 2. Baca nama tabel (mungkin dibungkus backtick ` atau petik)
        let _tabel_start = pos;
        let mut in_backtick = false;
        if pos < len && (bytes[pos] == b'`' || bytes[pos] == b'"') {
            in_backtick = true;
            pos += 1;
        }

        let nama_mulai = pos;
        while pos < len {
            let b = bytes[pos];
            if in_backtick {
                if b == b'`' || b == b'"' {
                    pos += 1;
                    break;
                }
            } else if (b as char).is_whitespace() || b == b'(' || b == b';' {
                break;
            }
            pos += 1;
        }
        let nama_tabel = text[nama_mulai..if in_backtick { pos - 1 } else { pos }].trim().to_lowercase();

        // 3. Cek apakah ada daftar kolom eksplisit: `(kolom1, kolom2, ...)`
        while pos < len && (bytes[pos] as char).is_whitespace() {
            pos += 1;
        }

        let mut kolom = None;
        if pos < len && bytes[pos] == b'(' {
            // Baca daftar kolom sampai tanda ')' penutup
            pos += 1;
            let kolom_start = pos;
            while pos < len && bytes[pos] != b')' {
                pos += 1;
            }
            if pos < len {
                let kolom_str = &text[kolom_start..pos];
                pos += 1; // lewati ')'
                let list_kolom: Vec<String> = kolom_str
                    .split(',')
                    .map(|k| k.trim().trim_matches('`').trim_matches('"').to_lowercase())
                    .collect();
                kolom = Some(list_kolom);
            }
        }

        // 4. Cari kata VALUES
        while pos < len {
            if Self::cek_keyword(text, pos, "VALUES") {
                pos += 6;
                break;
            }
            pos += 1;
        }

        // 5. Parse kumpulan baris tuple: (val1, val2), (val3, val4);
        let (rows, end_pos) = Self::parse_values_tuples(text, pos)?;

        Some((
            RawInsertStatement {
                tabel: nama_tabel,
                kolom,
                rows,
            },
            end_pos,
        ))
    }

    /// Parse sekumpulan tuple `(val1, val2), (val3, val4);` dengan penanganan kutipan string aman
    fn parse_values_tuples(text: &str, mut pos: usize) -> Option<(Vec<Vec<String>>, usize)> {
        let bytes = text.as_bytes();
        let len = bytes.len();
        let mut rows = Vec::new();

        while pos < len {
            // Lewati spasi dan koma antar-tuple
            while pos < len && ((bytes[pos] as char).is_whitespace() || bytes[pos] == b',') {
                pos += 1;
            }

            if pos >= len || bytes[pos] == b';' {
                pos += 1;
                break;
            }

            // Harus diawali tanda '('
            if bytes[pos] != b'(' {
                // Mungkin statement berakhir atau ada karakter lain
                break;
            }
            pos += 1; // lewati '('

            let mut current_row = Vec::new();
            let mut current_field = String::new();
            let mut in_quote = false;
            let mut quote_char = b'\'';
            let mut escaped = false;

            while pos < len {
                let b = bytes[pos];

                if escaped {
                    current_field.push(b as char);
                    escaped = false;
                    pos += 1;
                    continue;
                }

                if b == b'\\' {
                    current_field.push('\\');
                    escaped = true;
                    pos += 1;
                    continue;
                }

                if in_quote {
                    if b == quote_char {
                        // Cek double quote escape ('' dalam SQL)
                        if pos + 1 < len && bytes[pos + 1] == quote_char {
                            current_field.push(b as char);
                            pos += 2;
                            continue;
                        } else {
                            in_quote = false;
                            current_field.push(b as char);
                            pos += 1;
                            continue;
                        }
                    } else {
                        current_field.push(b as char);
                        pos += 1;
                        continue;
                    }
                } else {
                    if b == b'\'' || b == b'"' {
                        in_quote = true;
                        quote_char = b;
                        current_field.push(b as char);
                        pos += 1;
                        continue;
                    }

                    if b == b',' {
                        // Akhir dari sebuah field dalam baris
                        current_row.push(unquote_sql(&current_field));
                        current_field.clear();
                        pos += 1;
                        continue;
                    }

                    if b == b')' {
                        // Akhir dari satu baris tuple
                        current_row.push(unquote_sql(&current_field));
                        current_field.clear();
                        pos += 1;
                        rows.push(current_row);
                        break;
                    }

                    current_field.push(b as char);
                    pos += 1;
                }
            }
        }

        Some((rows, pos))
    }
}
