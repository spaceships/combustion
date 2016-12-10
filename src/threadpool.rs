use board::Board;
use moves::Move;
use util::ChessError;

use std::mem;
use std::thread;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Condvar;
use rand::{self, Rng};
use std::time::Duration;

type Worker = thread::JoinHandle<()>;

type JobId = usize;

struct Job {
    mv: Move,
    board: Board,
    depth: usize,
}

enum JobResult {
    Done { mv: Move, val: isize },
}

pub struct Threadpool {
    handles: Vec<Worker>,
    result_chan: Arc<Mutex<Receiver<JobResult>>>,
    jobs: Arc<JobQueue>,
    abort: Arc<Mutex<bool>>,
    main_signal: Arc<Condvar>,
    result_mutex: Arc<Mutex<Option<Result<(Move, isize), ChessError>>>>,
    thinking: Arc<Mutex<bool>>,
}

struct JobQueue {
    jobs: Mutex<Vec<Job>>,
    jobs_available: Condvar,
}

impl JobQueue {
    fn new() -> JobQueue {
        JobQueue {
            jobs: Mutex::new(Vec::new()),
            jobs_available: Condvar::new(),
        }
    }

    fn next_job(&self) -> Job {
        let job;
        loop {
            match self.jobs.lock().unwrap().pop() {
                Some(j) => {
                    job = j;
                    break;
                }
                None => { }
            }
            match self.jobs_available.wait(self.jobs.lock().unwrap()).unwrap().pop() {
                Some(j) => {
                    job = j;
                    break;
                }
                None => { }
            }
        }
        job
    }

    fn add_job(&self, job: Job) {
        self.jobs.lock().unwrap().push(job);
        self.jobs_available.notify_one();
    }
}

fn worker(s: Sender<JobResult>, q: Arc<JobQueue>, abort: Arc<Mutex<bool>>)
    -> Worker
{
    thread::spawn(move || {
        loop {
            // get next job
            match q.next_job() {
                Job { mv, board, depth } => {
                    let val = board.alpha_beta(depth, abort.clone());
                    s.send(JobResult::Done { mv: mv, val: val }).unwrap();
                }
            }
        }
    })
}

impl Threadpool {
    pub fn new(nthreads: usize, main_signal: Arc<Condvar>) -> Threadpool
    {
        let mut hs = Vec::new();
        let (result_tx, result_rx) = channel();
        let q = Arc::new(JobQueue::new());
        let abort = Arc::new(Mutex::new(false));

        for _ in 0..nthreads {
            hs.push(worker(result_tx.clone(), q.clone(), abort.clone()));
        }

        Threadpool {
            handles: hs,
            result_chan: Arc::new(Mutex::new(result_rx)),
            jobs: q,
            abort: abort,
            main_signal: main_signal,
            result_mutex: Arc::new(Mutex::new(None)),
            thinking: Arc::new(Mutex::new(false)),
        }
    }

    pub fn close(&mut self) {
        loop {
            match self.handles.pop() {
                Some(h) => h.join().unwrap(),
                None    => break,
            }
        }
    }

    pub fn abort(&self) {
        *self.abort.lock().unwrap() = true;
    }

    pub fn thinking(&self) -> bool {
        *self.thinking.lock().unwrap()
    }

    pub fn find_best_move(&mut self, b: &Board) {
        *self.thinking.lock().unwrap() = true;
        *self.abort.lock().unwrap() = false; // initialize abort flag
        let nmoves = match b.legal_moves() {
            Ok(moves) => {
                for mv in moves.iter() {
                    self.jobs.add_job(Job { mv: *mv, board: b.make_move(mv).unwrap(), depth: 5 });
                }
                moves.len()
            }

            Err(e) => {
                *self.result_mutex.lock().unwrap() = Some(Err(e));
                *self.thinking.lock().unwrap() = false;
                return;
            }
        };

        // bending over backwards to use a thread to clean up
        let rx = self.result_chan.clone();
        let result_mutex = self.result_mutex.clone();
        let thinking = self.thinking.clone();
        let main_signal = self.main_signal.clone();
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let mut best_score = isize::min_value();
            let mut best_move = None;
            for _ in 0..nmoves {
                match rx.lock().unwrap().recv().unwrap() {
                    JobResult::Done { mv, val } =>  {
                        if val > best_score || (val == best_score && rng.gen()) {
                            best_move = Some(mv);
                            best_score = val;
                        }
                    }
                }
            }
            *result_mutex.lock().unwrap() = Some(Ok((best_move.unwrap(), best_score)));

            *thinking.lock().unwrap() = false;
            main_signal.notify_all();
        });
    }

    pub fn has_result(&self) -> bool {
        self.result_mutex.lock().unwrap().is_some()
    }

    pub fn take_result(&self) -> Option<Result<(Move, isize), ChessError>> {
        mem::replace(&mut *self.result_mutex.lock().unwrap(), None)
    }

    pub fn abort_and_clear(&self) {
        self.abort();
        while *self.thinking.lock().unwrap() {
            thread::sleep(Duration::from_millis(50));
        }
        let _ = self.take_result();
    }
}
