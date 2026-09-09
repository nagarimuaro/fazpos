#!/bin/bash
set -e

echo "========================================================"
echo "  FAZPOS — Point of Sale & Retail Management System"
echo "========================================================"
echo "Starting via Docker on MacBook..."

docker compose up -d --build

echo "Waiting for display & noVNC server to initialize..."
sleep 4

echo "Opening FAZPOS in your browser..."
open "http://localhost:8080/index.html?autoconnect=true&resize=scale" || true

echo "========================================================"
echo "  FAZPOS is ready!"
echo "  URL: http://localhost:8080/index.html?autoconnect=true&resize=scale"
echo "  To stop: docker compose down"
echo "========================================================"
