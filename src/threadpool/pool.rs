use std::{
    sync::{
        Arc, Mutex,
        atomic::AtomicUsize,
        mpsc::{Receiver, Sender, channel},
    },
    thread::{self},
};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPoolData {
    job_receiver: Mutex<Receiver<Job>>,
    max_threads: AtomicUsize,
    active_threads: AtomicUsize,
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
            println!("Spawning thread");
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
}

fn spawn_threads(shared_data: Arc<ThreadPoolData>, thread_number: usize) {
    let builder = thread::Builder::new();
    let max_threads = shared_data
        .max_threads
        .load(std::sync::atomic::Ordering::Relaxed);
    let current_threads = shared_data
        .active_threads
        .load(std::sync::atomic::Ordering::Relaxed);
    //return early if there are enough threads
    if current_threads >= max_threads {
        return;
    };
    builder
        .spawn(move || {
            loop {
                println!("Thread {} spawned!", thread_number);
                //TODO: be less lazy about errors :p
                let message = {
                    let lock = shared_data.job_receiver.lock().unwrap();
                    lock.recv()
                };
                let job = match message {
                    Ok(job) => job,
                    //If no message was received then the threadpool was dropped
                    Err(..) => {
                        println!("No message received, dropping...");
                        break;
                    }
                };
                println!("Looking for jobs to do from thread: {}", thread_number);

                job();
            }
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
