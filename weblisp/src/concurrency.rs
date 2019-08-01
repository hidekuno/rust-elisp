/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

enum Message {
    NewJob(Job),
    Terminate,
}

type Job = Box<FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}
impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool { workers, sender }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        if let Err(e) = self.sender.send(Message::NewJob(job)) {
            eprintln!("send execute() err {}", e);
        }
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            if let Err(e) = self.sender.send(Message::Terminate) {
                eprintln!("send terminate() err {}", e);
            }
        }
        println!("shutdown all workers");

        for worker in &mut self.workers {
            println!("shutdown worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                if let Err(e) = thread.join() {
                    eprintln!("join() err {:?}", e);
                }
            }
        }
    }
}
//() is the unit type, analogous to a void return type in other languages.
pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = match receiver.lock().unwrap().recv() {
                Ok(message) => message,
                Err(e) => {
                    eprintln!("recv() err{}", e);
                    continue;
                }
            };

            match message {
                Message::NewJob(job) => {
                    println!("workder {} job; start.", id);
                    job();
                }
                Message::Terminate => {
                    println!("workder {} get a job; terminate.", id);
                    break;
                }
            }
            println!("workder {} job; finish.", id);
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}
