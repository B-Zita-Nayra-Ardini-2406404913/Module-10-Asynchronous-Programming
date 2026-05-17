use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

// =============================================
// BAGIAN 1: TimerFuture
// Ini adalah custom Future yang kita buat sendiri.
// Ia bisa menunggu (timer) tanpa memblokir thread.
// =============================================

pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

/// State yang di-share antara thread timer dan Future
struct SharedState {
    /// Apakah waktu tidur sudah selesai?
    completed: bool,
    /// Waker untuk task yang menunggu TimerFuture ini.
    /// Thread bisa menggunakan ini untuk memberitahu executor
    /// bahwa Future sudah bisa di-poll lagi.
    waker: Option<Waker>,
}

impl Future for TimerFuture {
    type Output = ();

    // Fungsi poll dipanggil oleh executor untuk mengecek apakah Future sudah selesai
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();

        if shared_state.completed {
            // Timer sudah selesai! Kembalikan Poll::Ready
            Poll::Ready(())
        } else {
            // Timer belum selesai.
            // Simpan waker agar thread timer bisa membangunkan task ini nanti
            shared_state.waker = Some(cx.waker().clone());
            // Kembalikan Poll::Pending (belum selesai, cek lagi nanti)
            Poll::Pending
        }
    }
}

impl TimerFuture {
    /// Buat TimerFuture baru yang akan selesai setelah durasi yang ditentukan.
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        // Spawn thread baru untuk menjalankan timer
        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            // Tidur selama durasi yang ditentukan (ini blocking, tapi di thread terpisah)
            thread::sleep(duration);
            let mut shared_state = thread_shared_state.lock().unwrap();
            // Tandai bahwa timer sudah selesai
            shared_state.completed = true;
            // Bangunkan task yang sedang menunggu (jika ada)
            if let Some(waker) = shared_state.waker.take() {
                waker.wake();
            }
        });

        TimerFuture { shared_state }
    }
}

// =============================================
// BAGIAN 2: Executor & Spawner
// Executor adalah yang menjalankan Future-Future kita.
// Spawner digunakan untuk mengirimkan task baru ke executor.
// =============================================

use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};
use std::sync::mpsc;

/// Task yang di-submit ke executor.
/// Setiap task berisi Future yang harus dijalankan.
struct Task {
    // Future yang sedang dijalankan.
    // Dibungkus Mutex agar thread-safe.
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    // Handle untuk menaruh task ini kembali ke antrian task
    task_sender: SyncSender<Arc<Task>>,
}

use std::sync::mpsc::SyncSender;

impl ArcWake for Task {
    // Ketika task ini di-wake (diberi tahu untuk melanjutkan),
    // taruh kembali ke antrian task agar executor menjalankannya lagi
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("too many tasks queued");
    }
}

/// Executor: membaca task dari channel dan menjalankannya
struct Executor {
    ready_queue: mpsc::Receiver<Arc<Task>>,
}

/// Spawner: mengirimkan task baru ke executor
#[derive(Clone)]
struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

/// Buat pasangan Executor dan Spawner baru
fn new_executor_and_spawner() -> (Executor, Spawner) {
    // Maksimum jumlah task yang menunggu di antrian
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = mpsc::sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}

impl Spawner {
    /// Kirimkan (spawn) future baru untuk dieksekusi oleh executor
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        // Kirim task ke antrian executor
        self.task_sender.send(task).expect("too many tasks queued");
    }
}

impl Executor {
    /// Jalankan semua task yang ada di antrian sampai habis
    fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            // Ambil future dari task
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                // Buat waker dari task ini
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&waker);
                // Poll future: apakah sudah selesai?
                if future.as_mut().poll(context).is_pending() {
                    // Belum selesai: taruh kembali future ke task
                    *future_slot = Some(future);
                }
                // Jika sudah selesai (Ready), kita buang future-nya
            }
        }
    }
}

// =============================================
// BAGIAN 3: main()
// =============================================

fn main() {
    let (executor, spawner) = new_executor_and_spawner();

    // Spawn task untuk mencetak sesuatu sebelum dan sesudah menunggu timer
    spawner.spawn(async {
        println!("Zita's Komputer: howdy!");
        // Tunggu timer future selesai setelah 2 detik.
        // Ini TIDAK memblokir thread utama.
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("Zita's Komputer: done!");
    });

    // Drop spawner agar executor tahu tidak akan ada task baru lagi.
    // Ini penting! Tanpa ini, executor.run() tidak akan pernah berhenti.
    drop(spawner);

    // Jalankan executor sampai semua task selesai.
    // Ini akan mencetak "howdy!", pause 2 detik, lalu "done!".
    executor.run();
}