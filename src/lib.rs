
// ============================================================
//  DAEGONICA SOFTWARE — lib.rs
//  Part of the Daegonica Software Rust Ecosystem
// ============================================================

//! # Daegonica Module: ThreadPool
//!
//! **Purpose:**
//! Provides a simple thread pool implementation for concurrent job execution in a server context.
//!
//! **Context:**
//! - Used in the server project to manage worker threads and distribute incoming jobs.
//!
//! **Responsibilities:**
//! - Owns worker threads and job queue.
//! - Handles job scheduling and graceful shutdown.
//! - Does NOT handle job prioritization or advanced scheduling.
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2025-12-04
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};


/// # ThreadPool
///
/// **Summary:**
/// Manages a fixed set of worker threads and a job queue for concurrent execution.
///
/// **Fields:**
/// - `workers`: Vector of worker threads.
/// - `sender`: Channel sender for job dispatching.
///
/// **Usage Example:**
/// ```rust
/// let pool = ThreadPool::new(4);
/// pool.execute(|| println!("Hello from a thread!"));
/// ```
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}


impl ThreadPool {

    /// # new
    ///
    /// **Purpose:**
    /// Creates a new thread pool with the specified number of worker threads.
    ///
    /// **Parameters:**
    /// - `size`: Number of worker threads to spawn.
    ///
    /// **Returns:**
    /// - A new `ThreadPool` instance.
    ///
    /// **Errors / Failures:**
    /// - Panics if `size` is zero.
    ///
    /// **Examples:**
    /// ```rust
    /// let pool = ThreadPool::new(4);
    /// ```
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// # execute
    ///
    /// **Purpose:**
    /// Sends a job (closure) to the thread pool for execution by a worker thread.
    ///
    /// **Parameters:**
    /// - `f`: Closure to execute. Must be `FnOnce() + Send + 'static`.
    ///
    /// **Returns:**
    /// None.
    ///
    /// **Errors / Failures:**
    /// - Panics if the sender channel is closed or job cannot be sent.
    ///
    /// **Examples:**
    /// ```rust
    /// pool.execute(|| println!("Hello from a thread!"));
    /// ```
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}


impl Drop for ThreadPool {
    /// # drop
    ///
    /// **Purpose:**
    /// Gracefully shuts down the thread pool and joins all worker threads.
    ///
    /// **Parameters:**
    /// None.
    ///
    /// **Returns:**
    /// None.
    ///
    /// **Errors / Failures:**
    /// - Panics if a worker thread cannot be joined.
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}


/// # Job
///
/// **Summary:**
/// Type alias for boxed closures that can be sent to worker threads for execution.
type Job = Box<dyn FnOnce() + Send + 'static>;


/// # Worker
///
/// **Summary:**
/// Represents a single worker thread in the thread pool.
///
/// **Fields:**
/// - `id`: Worker thread identifier.
/// - `thread`: Handle to the spawned thread.
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}


impl Worker {
    /// # new
    ///
    /// **Purpose:**
    /// Spawns a new worker thread that waits for and executes jobs from the job queue.
    ///
    /// **Parameters:**
    /// - `id`: Worker thread identifier.
    /// - `receiver`: Shared receiver for job queue.
    ///
    /// **Returns:**
    /// - A new `Worker` instance with a running thread.
    ///
    /// **Errors / Failures:**
    /// - Panics if thread spawning fails.
    ///
    /// **Examples:**
    /// ```rust
    /// let worker = Worker::new(0, receiver);
    /// ```
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}