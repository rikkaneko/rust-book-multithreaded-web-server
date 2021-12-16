use std::thread;
use std::thread::JoinHandle;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Instant;

pub struct ThreadPool {
	workers: Vec<Worker>,
	sender: mpsc::Sender<Message>
}

pub struct Worker {
	id: usize,
	thread: Option<JoinHandle<()>>
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
	Job(Job),
	Terminate
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
	
	pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
		let job = Box::new(f);
		self.sender.send(Message::Job(job)).unwrap();
	}
}

impl Drop for ThreadPool {
	fn drop(&mut self) {
		for worker in &self.workers {
			println!("Send termination signal to Work {}", worker.id);
			self.sender.send(Message::Terminate).unwrap();
		}
		
		for worker in &mut self.workers {
			if let Some(thread) = worker.thread.take() {
				thread.join().unwrap();
			}
			println!("Worker {} terminated.", worker.id);
		}
	}
}

impl Worker {
	fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
		println!("Worker {} created.", id);
		let thread = thread::spawn(move || loop {
			let message = receiver.lock().unwrap().recv().unwrap();
			match message {
				Message::Job(job) => {
					println!("Worker {}: Task started.", id);
					let start = Instant::now();
					job();
					let duration = start.elapsed();
					println!("Worker {}: Task ended with {:.2}s elapsed.", id, duration.as_secs_f32());
				},
				
				Message::Terminate => {
					println!("Worker {} is terminating.", id);
					return
				}
			}
		});
		
		Worker { id, thread: Some(thread) }
	}
}
