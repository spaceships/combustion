extern crate combustion;
extern crate num_cpus;

use combustion::board::Board;
use combustion::threadpool::Threadpool;

use std::time::Instant;
use std::sync::{Arc, Condvar, Mutex};

// sees how long it takes to solve this tactic
fn main() {
    let b = Board::from_fen("1K6/2P5/1p3P2/1k2P3/1qnP1B2/3Q4/8/8 b - - 0 1").unwrap();
    println!("{}", b);
    let main_signal = Arc::new(Condvar::new());
    let main_mutex = Mutex::new(());
    let mut pool = Threadpool::new(num_cpus::get(), main_signal.clone());
    let start = Instant::now();
    pool.find_best_move(&b, 6);
    println!("started search...");
    let _ = main_signal.wait(main_mutex.lock().unwrap()).unwrap();
    let (mv, score) = pool.take_result().unwrap().unwrap();
    println!("finished search: move={} score={} took={}s", mv, score, start.elapsed().as_secs());
}
