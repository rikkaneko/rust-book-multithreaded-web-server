/*
 * This file is part of rust-book-multithreaded-web-server.
 * Copyright (c) 2021-2021 Joe Ma <rikkaneko23@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

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
