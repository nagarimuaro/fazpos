use fazpos::db::Database;
use fazpos::domain::cabang::Cabang;
use fazpos::domain::operator::DOperator;
use fazpos::repository::cabang_repo::CabangRepo;
use fazpos::repository::operator_repo::OperatorRepo;

#[test]
fn test_admin_permanen_dan_hanya_bisa_ubah_password() {
    let db = Database::buka_in_memory().expect("Buka db in-memory");
    let cabang_repo = CabangRepo::new(db.conn());
    let cabang = Cabang::baru("CAB-ADM", "Toko Cabang Admin", true);
    cabang_repo.simpan_cabang(&cabang).unwrap();

    let op_repo = OperatorRepo::new(db.conn());

    // 1. Buat user admin permanen pertama kali
    let pass_awal = OperatorRepo::hash_password("admin");
    let admin_op = DOperator::baru(&cabang.id, "admin", "Administrator", pass_awal, "admin");
    op_repo.simpan(&admin_op).unwrap();

    // Verifikasi login admin dengan password default
    let login_default = op_repo.verifikasi(&cabang.id, "admin", "admin").unwrap();
    assert!(login_default.is_some());
    let adm = login_default.unwrap();
    assert_eq!(adm.kode, "admin");
    assert_eq!(adm.nama, "Administrator");
    assert_eq!(adm.role, "admin");

    // 2. Coba hapus user admin -> HARUS GAGAL
    let hapus_res = op_repo.hapus(&adm.id);
    assert!(hapus_res.is_err(), "User admin permanen tidak boleh bisa dihapus");

    // 3. Coba ubah username/kode admin ke kode lain -> HARUS GAGAL
    let mut op_ubah_kode = adm.clone();
    op_ubah_kode.kode = "superuser".to_string();
    let ubah_kode_res = op_repo.simpan(&op_ubah_kode);
    assert!(ubah_kode_res.is_err(), "Username admin permanen tidak boleh bisa diubah");

    // 4. Coba ubah nama/role admin via simpan -> Nama dan role TIDAK berubah, tetap Administrator & admin
    let mut op_ubah_nama = adm.clone();
    op_ubah_nama.nama = "Hacker".to_string();
    op_ubah_nama.role = "kasir".to_string();
    op_ubah_nama.password_hash = OperatorRepo::hash_password("admin_baru_123");
    op_repo.simpan(&op_ubah_nama).unwrap();

    let adm_setelah_simpan = op_repo.cari_by_id(&adm.id).unwrap().unwrap();
    assert_eq!(adm_setelah_simpan.nama, "Administrator", "Nama admin tidak boleh berubah");
    assert_eq!(adm_setelah_simpan.role, "admin", "Role admin tidak boleh berubah");

    // Verifikasi password baru aktif
    let login_pass_baru = op_repo.verifikasi(&cabang.id, "admin", "admin_baru_123").unwrap();
    assert!(login_pass_baru.is_some(), "Password baru admin harus bisa login");

    // Password lama tidak bisa login lagi
    let login_pass_lama = op_repo.verifikasi(&cabang.id, "admin", "admin").unwrap();
    assert!(login_pass_lama.is_none(), "Password lama tidak boleh bisa login lagi");

    // 5. Ubah password via ubah_password method
    let ubah_ok = op_repo.ubah_password(&cabang.id, "admin", "admin_rahasia").unwrap();
    assert!(ubah_ok);
    let login_rahasia = op_repo.verifikasi(&cabang.id, "admin", "admin_rahasia").unwrap();
    assert!(login_rahasia.is_some());
}
