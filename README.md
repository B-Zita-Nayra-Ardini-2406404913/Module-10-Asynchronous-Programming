Nama: Zita Nayra Ardini
NPM: 2406404913
Kelas: Pemrograman Lanjut B

# TUTORIAL TIMER
## Eksperimen 1.2: Memahami Cara Kerja Executor
### Screenshot:
![img.png](async-timer/img.png)

### Penjelasan:
Ketika saya menambahkan println! setelah spawner.spawn(), ternyata output "hey hey!" muncul lebih dulu sebelum "howdy!". Hal ini terjadi karena spawner.spawn() hanya mendaftarkan future ke antrian executor, tetapi tidak langsung menjalankannya. Eksekusi sebenarnya baru terjadi ketika executor.run() dipanggil. Oleh karena itu, kode yang berada di luar blok async (yaitu println!("hey hey!")) dieksekusi secara sinkronus terlebih dahulu. Ini membuktikan bahwa future di Rust bersifat lazy: mereka tidak berjalan tanpa executor yang memanggil poll. Setelah executor.run() dijalankan, executor mengambil task dari antrian, memanggil poll, dan mencetak "howdy!". Kemudian timer berjalan 2 detik, dan setelah selesai mencetak "done!". Urutan ini menunjukkan pemisahan antara penjadwalan (spawn) dan eksekusi (poll) dalam model async Rust.

## Eksperimen 1.3: Multiple Spawn dan Menghapus drop
### Screenshot tanpa drop:
![img1.png](async-timer/img1.png)
Program mencetak semua "howdy" dan "done", lalu hang (tidak berhenti). Harus dihentikan manual (Ctrl+C).

### Screenshot dengan drop:
![img2.png](async-timer/img2.png)

### Penjelasan:
- Multiple spawn:
> Executor memanggil poll untuk setiap task secara bergantian. Pada polling pertama, semua task langsung mencetak "howdy", memulai timer masing-masing, lalu mengembalikan Pending. Setelah 2 detik, semua timer selesai hampir bersamaan, executor melanjutkan polling dan mencetak "done". Ini menunjukkan konkurensi yaitu tiga task berjalan overlapped dalam rentang waktu yang sama.

- Menghapus drop:
> Executor menggunakan while let Ok(task) = ready_queue.recv(). Fungsi recv() akan menunggu selama channel masih terbuka. Channel ditutup hanya ketika semua SyncSender (termasuk spawner) di-drop. Karena spawner tidak di-drop, executor terus menunggu task baru meskipun antrian kosong. Akibatnya program tidak pernah berhenti.
