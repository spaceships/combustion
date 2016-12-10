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

enum Message {
    Aborted,
}

pub struct Threadpool {
    handles: Vec<Worker>,
    result_chan: Receiver<JobResult>,
    msg_chan: Receiver<Message>,
    jobs: Arc<JobQueue>,
    abort: Arc<Mutex<bool>>,
    nthreads: usize,
    main_signal: Arc<Condvar>,
    result_mutex: Mutex<Option<Result<(Move, isize), ChessError>>>,
    thinking: Mutex<bool>,
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
            {
                match self.jobs.lock().unwrap().pop() {
                    Some(j) => {
                        job = j;
                        break;
                    }
                    None => { }
                }
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

    fn reset(&self) {
        self.jobs.lock().unwrap().clear();
    }
}

fn worker(s: Sender<JobResult>, m: Sender<Message>, q: Arc<JobQueue>, abort: Arc<Mutex<bool>>)
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
            if *abort.lock().unwrap() {
                m.send(Message::Aborted).unwrap();
            }
        }
    })
}

impl Threadpool {
    pub fn new(nthreads: usize, main_signal: Arc<Condvar>) -> Threadpool
    {
        let mut hs = Vec::new();
        let (result_tx, result_rx) = channel();
        let (msg_tx, msg_rx) = channel();
        let q = Arc::new(JobQueue::new());
        let abort = Arc::new(Mutex::new(false));

        for _ in 0..nthreads {
            hs.push(worker(result_tx.clone(), msg_tx.clone(), q.clone(), abort.clone()));
        }

        Threadpool {
            nthreads: nthreads,
            handles: hs,
            result_chan: result_rx,
            msg_chan: msg_rx,
            jobs: q,
            abort: abort,
            main_signal: main_signal,
            result_mutex: Mutex::new(None),
            thinking: Mutex::new(false),
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

    pub fn abort(&mut self) {
        *self.abort.lock().unwrap() = true;
        self.jobs.reset();
        for _ in 0..self.nthreads {
            match self.msg_chan.recv().unwrap() {
                Message::Aborted => {}
            }
        }
        *self.abort.lock().unwrap() = false;
        self.main_signal.notify_all();
    }

    pub fn thinking(&self) -> bool {
        *self.thinking.lock().unwrap()
    }

    pub fn find_best_move(&mut self, b: &Board) {
        // notify the caller somehow when it is found
        *self.thinking.lock().unwrap() = true;
        match b.legal_moves() {
            Ok(moves) => {
                for mv in moves.iter() {
                    self.jobs.add_job(Job { mv: *mv, board: b.make_move(mv).unwrap(), depth: 5 });
                }

                // TODO: this is still blocking
                let mut rng = rand::thread_rng();
                let mut best_score = isize::min_value();
                let mut best_move = None;
                for _ in 0..moves.len() {
                    match self.result_chan.recv().unwrap() {
                        JobResult::Done { mv, val } =>  {
                            if val > best_score || (val == best_score && rng.gen()) {
                                best_move = Some(mv);
                                best_score = val;
                            }
                        }
                    }
                }
                *self.result_mutex.lock().unwrap() = Some(Ok((best_move.unwrap(), best_score)));
            }

            Err(e) => {
                *self.result_mutex.lock().unwrap() = Some(Err(e));
            }
        }
        *self.thinking.lock().unwrap() = false;
        self.main_signal.notify_all();
    }

    pub fn has_result(&self) -> bool {
        self.result_mutex.lock().unwrap().is_some()
    }

    pub fn take_result(&self) -> Option<Result<(Move, isize), ChessError>> {
        mem::replace(&mut *self.result_mutex.lock().unwrap(), None)
    }
}
