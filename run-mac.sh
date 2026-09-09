#!/usr/bin/env bash

echo "========================================================"
echo "  FAZPOS — Point of Sale & Retail Management System"
echo "========================================================"

# Pastikan direktori kerja adalah folder proyek
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd "$DIR"

# 1. Cek apakah Docker terpasang
if ! command -v docker &> /dev/null; then
    echo "⚠️  Docker belum terpasang di MacBook ini."
    if command -v cargo &> /dev/null; then
        echo "💡 Terdeteksi Rust/Cargo! Menjalankan FAZPOS secara native..."
        cargo run
        exit 0
    else
        echo "❌ Silakan pasang Docker Desktop (https://www.docker.com/products/docker-desktop/)"
        echo "   atau pasang Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
fi

# 2. Cek apakah Docker daemon aktif
if ! docker info &> /dev/null; then
    echo "⏳ Docker Desktop belum menyala. Membuka aplikasi Docker..."
    open -a Docker 2>/dev/null || true
    echo "Menunggu Docker siap (maksimal 30 detik)..."
    READY=0
    for i in {1..30}; do
        if docker info &> /dev/null; then
            READY=1
            break
        fi
        sleep 1
        printf "."
    done
    echo ""
    if [ $READY -eq 0 ]; then
        echo "❌ Docker Desktop belum siap. Buka aplikasi 'Docker' dari Launchpad/Applications,"
        echo "   tunggu hingga icon Docker di menu bar atas selesai memuat, lalu jalankan script ini lagi."
        exit 1
    fi
fi

# 3. Tentukan perintah compose (docker compose vs docker-compose)
if docker compose version &> /dev/null; then
    COMPOSE_CMD="docker compose"
elif command -v docker-compose &> /dev/null; then
    COMPOSE_CMD="docker-compose"
else
    echo "❌ Perintah docker compose tidak ditemukan."
    exit 1
fi

# 4. Jalankan kontainer
echo "🚀 Membangun dan menjalankan kontainer FAZPOS..."
mkdir -p data
$COMPOSE_CMD up -d --build

# 5. Tunggu server noVNC siap
echo "⏳ Menunggu display GUI & web server siap..."
sleep 4

# 6. Buka otomatis di browser default MacBook
URL="http://localhost:8080/index.html?autoconnect=true&resize=scale"
echo "🌐 Membuka FAZPOS di browser ($URL)..."
open "$URL" 2>/dev/null || xdg-open "$URL" 2>/dev/null || true

echo "========================================================"
echo "  ✅ FAZPOS AKTIF!"
echo "  Akses Web GUI: $URL"
echo "  Untuk menghentikan: $COMPOSE_CMD down"
echo "========================================================"