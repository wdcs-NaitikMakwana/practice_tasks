use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

#[derive(Debug, PartialEq, Clone, Copy)]
enum State {
    Init,
    Downloading,
    Compressing,
    Pushing,
    Completed,
}

#[derive(Clone)]
struct PushWorker {
    state: State,
    download_urls:  Vec<String>,
    downloaded_count: usize,
    is_compress: bool,
    is_pushed: bool,
}

impl PushWorker {
    fn new() -> Self {
        Self {
            state: State::Init,
            download_urls: Vec::new(),
            downloaded_count: 0,
            is_compress: false,
            is_pushed: false,
        }
    }

    fn transition_state(&mut self, new_state: State) {
        self.state = new_state;
    }


    fn start_downloading(&mut self, urls: &Vec<String>, barrier: Arc<Barrier>, n:usize ) {
        self.download_urls = urls.clone();
        self.state = State::Downloading;
        println!("Node {:?} starting downloading.", n);

        // Simulate downloading asynchronously
        let mut handles = vec![];
        for url in self.download_urls.iter().cloned() {
            let barrier_clone = Arc::clone(&barrier);
            let mut worker = self.clone();
            let handle = thread::spawn(move || {
                // Simulate download time
                thread::sleep(Duration::from_secs(5));
                println!("Node {:?} downloaded {}", n, url);
                worker.downloaded_count += 1;
                barrier_clone.wait();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
        self.transition_state(State::Compressing);
        thread::sleep(Duration::from_secs(5));

    }

    fn start_compressing(&mut self, barrier: Arc<Barrier>, n:usize) {
        self.is_compress = true;
        println!("Node {:?} compressing...", n);
        thread::sleep(Duration::from_secs(5));
        println!("Node {:?} compression completed.", n);
        self.transition_state(State::Pushing);
        barrier.wait(); 
    }


    fn start_pushing(&mut self, barrier: Arc<Barrier>, n:usize) {
        self.is_pushed = true;
        println!("Node {:?} pushing...", n);
        thread::sleep(Duration::from_secs(5));
        println!("Node {:?} pushing completed.", n);
        self.transition_state(State::Completed);
        barrier.wait(); 
    }
}


fn main() {
    let num_nodes = 3; // Number of nodes
    let barrier = Arc::new(Barrier::new(num_nodes));
    let mut threads = vec![];

    for n in 0..num_nodes {
        let barrier = Arc::clone(&barrier);
        let urls = vec![
            String::from("http://example.com/file1"),
            String::from("http://example.com/file2"),
            String::from("http://example.com/file3"),
        ];

        let thread_handle = thread::spawn(move || {
            let mut worker = PushWorker::new();
           
           //Downloading
            worker.start_downloading(&urls, barrier.clone(),n);
            barrier.wait();

            //Compressing
            if worker.state == State::Compressing && worker.is_compress == false {
                worker.start_compressing(barrier.clone(), n);
            }
            barrier.wait(); 
        

            //Pushing
            if worker.state == State::Pushing && worker.is_pushed == false {
                worker.start_pushing(barrier.clone(),n);
            }
            barrier.wait(); 
        
        });

        threads.push(thread_handle);
    }

    for thread in threads {
        thread.join().unwrap();
    }

    println!("All tasks completed successfully!");
}
