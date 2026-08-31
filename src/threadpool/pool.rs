use std::{
    sync::{
        Arc, Mutex,
        atomic::{
            AtomicUsize,
            Ordering::{Acquire, Relaxed, SeqCst},
        },
        mpsc::{Receiver, Sender, channel},
    },
    thread::{self},
};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct PanicWatch<'a> {
    active: bool,
    shared_data: &'a Arc<ThreadPoolData>,
}

impl<'a> PanicWatch<'a> {
    fn new(shared_data: &'a Arc<ThreadPoolData>) -> PanicWatch<'a> {
        PanicWatch {
            shared_data,
            active: true,
        }
    }
    fn cancel(mut self) {
        self.active = false;
    }
}

impl<'a> Drop for PanicWatch<'a> {
    //If a thread panicks while the PanicWatcher is active, it automatically adds a new thread to
    //the pool
    fn drop(&mut self) {
        if !self.active {
            return;
        };
        self.shared_data.active_threads.fetch_sub(1, SeqCst);
        if thread::panicking() {
            self.shared_data.panicked_threads.fetch_add(1, SeqCst);
        };
        //spawn_threads has checking for active_threads against max_threads
        spawn_threads(
            self.shared_data.clone(),
            self.shared_data.active_threads.load(Relaxed)
                + self.shared_data.panicked_threads.load(Relaxed)
                + 1,
        );
    }
}

struct ThreadPoolData {
    job_receiver: Mutex<Receiver<Job>>,
    max_threads: AtomicUsize,
    active_threads: AtomicUsize,
    panicked_threads: AtomicUsize,
}

pub struct ThreadPool {
    jobs: Sender<Job>,
    shared_data: Arc<ThreadPoolData>,
}

impl ThreadPool {
    pub fn new(thread_count: usize) -> Self {
        let (rx, tx) = channel::<Job>();
        let shared_data = Arc::new(ThreadPoolData {
            job_receiver: Mutex::new(tx),
            max_threads: AtomicUsize::new(thread_count),
            active_threads: AtomicUsize::new(0),
            panicked_threads: AtomicUsize::new(0),
        });
        ThreadPool {
            jobs: rx,
            shared_data,
        }
    }

    pub fn build(&self) {
        for thread_number in 0..self
            .shared_data
            .max_threads
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            if cfg!(feature = "debug-print") {
                println!("Spawning thread: {}", thread_number);
            }

            spawn_threads(self.shared_data.clone(), thread_number)
        }
    }

    pub fn add_job<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.jobs
            .send(Box::new(job))
            .expect("Unable to send job into queue")
    }

    pub fn panic_count(&self) -> usize {
        self.shared_data.panicked_threads.load(Relaxed)
    }
}

fn spawn_threads(shared_data: Arc<ThreadPoolData>, thread_number: usize) {
    let builder = thread::Builder::new();
    let max_threads = shared_data.max_threads.load(Relaxed);
    let current_threads = shared_data.active_threads.load(Acquire);
    //return early if there are enough threads
    if current_threads >= max_threads {
        return;
    };
    shared_data.active_threads.fetch_add(1, SeqCst);
    builder
        .spawn(move || {
            let watcher = PanicWatch::new(&shared_data);
            loop {
                if cfg!(feature = "debug-print") {
                    println!("Thread {} spawned!", thread_number);
                }
                //TODO: be less lazy about errors :p
                let message = {
                    let lock = shared_data.job_receiver.lock().unwrap();
                    lock.recv()
                };
                let job = match message {
                    Ok(job) => job,
                    //If The connection is closed, drop the thread
                    Err(..) => {
                        if cfg!(feature = "debug-print") {
                            println!("Connection closed, dropping...");
                        }
                        break;
                    }
                };

                if cfg!(feature = "debug-print") {
                    println!("Got a job to do for thread: {}", thread_number);
                }

                job();
            }
            watcher.cancel();
        })
        .unwrap();
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn task_with_message() {
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }

        let (rx, tx) = channel::<i32>();
        let pool = ThreadPool::new(1);
        pool.build();
        pool.add_job(move || {
            let num = add(5, 10);
            rx.send(num).unwrap()
        });

        assert_eq!(15, tx.recv().unwrap());
    }
}
