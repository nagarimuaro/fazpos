use crate::db::Database;
use crate::importer::mappers::barang_mapper::BarangMapper;
use crate::importer::mappers::operator_mapper::OperatorMapper;
use crate::importer::mappers::pelanggan_mapper::PelangganMapper;
use crate::importer::sql_parser::{RawInsertStatement, SqlDumpParser};
use crate::repository::barang_repo::BarangRepo;
use rusqlite::params;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct ImportReport {
    pub total_barang_sukses: usize,
    pub total_pelanggan_sukses: usize,
    pub total_operator_sukses: usize,
    pub total_gagal: usize,
    pub errors: Vec<String>,
}

pub struct ImportService;

impl ImportService {
    /// Import data dari path file .sql dump legacy
    pub fn import_dari_file<P: AsRef<Path>>(
        db: &mut Database,
        cabang_id: &str,
        file_path: P,
    ) -> Result<ImportReport, String> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Gagal membaca file SQL dump: {}", e))?;
        Self::import_dari_teks(db, cabang_id, &content)
    }

    /// Import data langsung dari string teks SQL dump
    pub fn import_dari_teks(
        db: &mut Database,
        cabang_id: &str,
        sql_content: &str,
    ) -> Result<ImportReport, String> {
        let mut report = ImportReport::default();

        // 1. Ekstrak seluruh statement INSERT
        let statements = SqlDumpParser::parse_dump(sql_content);

        // 2. Eksekusi import dengan memprioritaskan tabel produk (dbarang)
        // Kelompokkan per nama tabel
        let mut stmts_barang = Vec::new();
        let mut stmts_pelanggan = Vec::new();
        let mut stmts_operator = Vec::new();
        let mut stmts_lain = Vec::new();

        for s in statements {
            match s.tabel.as_str() {
                "dbarang" => stmts_barang.push(s),
                "dpelanggan" => stmts_pelanggan.push(s),
                "doperator" => stmts_operator.push(s),
                _ => stmts_lain.push(s),
            }
        }

        // Jalankan Prioritas #1: dbarang
        for s in stmts_barang {
            Self::proses_barang(db, cabang_id, &s, &mut report);
        }

        // Jalankan Prioritas #2: dpelanggan
        for s in stmts_pelanggan {
            Self::proses_pelanggan(db, cabang_id, &s, &mut report);
        }

        // Jalankan Prioritas #3: doperator
        for s in stmts_operator {
            Self::proses_operator(db, cabang_id, &s, &mut report);
        }

        Ok(report)
    }

    fn proses_barang(
        db: &Database,
        cabang_id: &str,
        stmt: &RawInsertStatement,
        report: &mut ImportReport,
    ) {
        let repo = BarangRepo::new(db.conn());
        for (i, row) in stmt.rows.iter().enumerate() {
            match BarangMapper::petakan(cabang_id, stmt.kolom.as_deref(), row) {
                Ok(barang) => {
                    if let Err(e) = repo.simpan(&barang) {
                        report.total_gagal += 1;
                        report.errors.push(format!(
                            "Gagal simpan barang [baris {}]: {}",
                            i + 1,
                            e
                        ));
                    } else {
                        report.total_barang_sukses += 1;
                    }
                }
                Err(err) => {
                    report.total_gagal += 1;
                    report.errors.push(format!(
                        "Gagal konversi barang [baris {}]: {}",
                        i + 1,
                        err
                    ));
                }
            }
        }
    }

    fn proses_pelanggan(
        db: &Database,
        cabang_id: &str,
        stmt: &RawInsertStatement,
        report: &mut ImportReport,
    ) {
        for (i, row) in stmt.rows.iter().enumerate() {
            match PelangganMapper::petakan(cabang_id, stmt.kolom.as_deref(), row) {
                Ok(p) => {
                    let res = db.conn().execute(
                        r#"
                        INSERT INTO dpelanggan (
                            id, cabang_id, kode, nama, alamat, telepon, plafonpiutang,
                            poin_saldo, id_kartu, is_aktif, sync_status, created_at, updated_at
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'pending', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                        ON CONFLICT(cabang_id, kode) DO UPDATE SET
                            nama = excluded.nama,
                            alamat = excluded.alamat,
                            telepon = excluded.telepon,
                            plafonpiutang = excluded.plafonpiutang,
                            poin_saldo = excluded.poin_saldo,
                            id_kartu = excluded.id_kartu,
                            is_aktif = excluded.is_aktif,
                            updated_at = CURRENT_TIMESTAMP;
                        "#,
                        params![
                            p.id, p.cabang_id, p.kode, p.nama, p.alamat, p.telepon,
                            p.plafonpiutang, p.poin_saldo, p.id_kartu, if p.is_aktif { 1 } else { 0 }
                        ],
                    );
                    if let Err(e) = res {
                        report.total_gagal += 1;
                        report.errors.push(format!("Gagal simpan pelanggan [baris {}]: {}", i + 1, e));
                    } else {
                        report.total_pelanggan_sukses += 1;
                    }
                }
                Err(err) => {
                    report.total_gagal += 1;
                    report.errors.push(format!("Gagal konversi pelanggan [baris {}]: {}", i + 1, err));
                }
            }
        }
    }

    fn proses_operator(
        db: &Database,
        cabang_id: &str,
        stmt: &RawInsertStatement,
        report: &mut ImportReport,
    ) {
        for (i, row) in stmt.rows.iter().enumerate() {
            match OperatorMapper::petakan(cabang_id, stmt.kolom.as_deref(), row) {
                Ok(op) => {
                    let res = db.conn().execute(
                        r#"
                        INSERT INTO doperator (
                            id, cabang_id, kode, nama, password_hash, role, is_aktif, created_at, updated_at
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                        ON CONFLICT(cabang_id, kode) DO UPDATE SET
                            nama = excluded.nama,
                            password_hash = excluded.password_hash,
                            role = excluded.role,
                            is_aktif = excluded.is_aktif,
                            updated_at = CURRENT_TIMESTAMP;
                        "#,
                        params![
                            op.id, op.cabang_id, op.kode, op.nama, op.password_hash,
                            op.role, if op.is_aktif { 1 } else { 0 }
                        ],
                    );
                    if let Err(e) = res {
                        report.total_gagal += 1;
                        report.errors.push(format!("Gagal simpan operator [baris {}]: {}", i + 1, e));
                    } else {
                        report.total_operator_sukses += 1;
                    }
                }
                Err(err) => {
                    report.total_gagal += 1;
                    report.errors.push(format!("Gagal konversi operator [baris {}]: {}", i + 1, err));
                }
            }
        }
    }
}
