# 🦀 MDWeave

Sebuah *tool* berbasis Rust untuk mengekstrak dan mengonversi dokumen (seperti PDF dan file office lainnya) menjadi format teks/Markdown. Proyek ini dibangun sebagai alternatif yang cepat dan aman dengan memanfaatkan ekosistem Rust, terinspirasi oleh proyek [MarkItDown](https://github.com/microsoft/markitdown) dari Microsoft.

## 🚀 Fitur

- Penguraian PDF berkinerja tinggi menggunakan [Pdfium](https://pdfium.googlesource.com/pdfium/).
- Pengenalan Karakter Optik (OCR) menggunakan **Tesseract**.
- Ditulis sepenuhnya dalam Rust untuk kecepatan dan keamanan memori.

## 📋 Prasyarat & Instalasi

Sebelum menjalankan proyek ini, Anda harus menginstal **Rust** dan beberapa dependensi eksternal, terutama Tesseract OCR.

### 1. Instalasi Rust
Jika Anda belum menginstal Rust, silakan instal melalui [rustup](https://rustup.rs/):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Instalasi Tesseract OCR (Wajib)
Proyek ini membutuhkan Tesseract agar fitur ekstraksi teks dari gambar atau PDF berbasis gambar dapat berjalan.

**Untuk Pengguna Windows:**
1. Unduh installer Tesseract dari UB-Mannheim: [Tesseract at UB-Mannheim](https://github.com/UB-Mannheim/tesseract/wiki).
2. Jalankan installer dan ikuti petunjuknya. Secara *default*, Tesseract akan diinstal di `C:\Program Files\Tesseract-OCR`.
3. **Setting System Path (Penting):**
   - Buka Start Menu Windows, ketik **Environment Variables**, lalu pilih *Edit the system environment variables*.
   - Di tab *Advanced*, klik tombol **Environment Variables...** di bagian bawah.
   - Pada bagian *System variables* (atau *User variables*), cari variabel bernama **Path**, lalu klik **Edit**.
   - Klik **New**, dan tambahkan *path* lokasi instalasi Tesseract (biasanya `C:\Program Files\Tesseract-OCR`).
   - Klik **OK** di semua jendela untuk menyimpan pengaturan.
   - **Tutup dan buka kembali terminal/Command Prompt Anda** agar perubahan Path terbaca.
4. Verifikasi instalasi dengan menjalankan perintah berikut di terminal:
   ```bash
   tesseract --version
   ```

**Untuk Pengguna macOS:**
```bash
brew install tesseract
```

**Untuk Pengguna Linux (Ubuntu/Debian):**
```bash
sudo apt-get install tesseract-ocr
```

### 3. Pdfium
Proyek ini menggunakan *binding* Rust untuk Pdfium (misalnya melalui *crate* `pdfium-render`). Tergantung pada sistem operasi Anda, Anda mungkin perlu mengunduh binari `pdfium` yang telah dikompilasi (seperti `pdfium.dll` untuk Windows atau `libpdfium.so` untuk Linux) dan meletakkannya di *root* direktori proyek ini atau menambahkannya ke *path* sistem. *(Sudah kami sertakan, bisa anda cek dihalaman instalasi maupun di folder libs repository)*

## 🛠️ Penggunaan

Kloning repositori ini dan jalankan menggunakan `cargo`:

```bash
git clone https://github.com/hasansulthoni/mdweave.git
cd mdweave

# Build project
cargo build --release

# Jalankan project
cargo run
```

## 📄 Lisensi

Proyek ini didistribusikan di bawah Lisensi MIT. Lihat file `LICENSE` untuk informasi lebih lanjut.

## 🙏 Acknowledgments (Penghargaan)

- **[MarkItDown (Microsoft)](https://github.com/microsoft/markitdown)**: Proyek ini sangat terinspirasi dari fungsionalitas dan konsep MarkItDown milik Microsoft.
- Pembuatan kode dalam proyek ini turut dibantu oleh *Artificial Intelligence* (AI) untuk mempercepat pengembangan.
- Terima kasih kepada komunitas *open-source* Rust, pengembang [Tesseract](https://github.com/tesseract-ocr/tesseract), dan pemelihara library [Pdfium](https://pdfium.googlesource.com/pdfium/).