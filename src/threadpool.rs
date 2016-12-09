use board::Board;
use moves::Move;
use util::ChessError;

use std::thread;
use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Condvar;
use std::io::Write;
use rand::{self, Rng};

type Worker = thread::JoinHandle<()>;

type JobId = usize;

struct Job {
    mv: Move,
    board: Board,
    depth: usize,
}

enum Message {
    Abort,
}

pub struct Threadpool {
    handles: Vec<Worker>,
    sends: Vec<Sender<Message>>,
    receive: Receiver<(Move, isize)>,
    jobs: Arc<JobQueue>,
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
}

fn worker(threadnum: usize, r: Receiver<Message>, s: Sender<(Move, isize)>, q: Arc<JobQueue>)
    -> Worker
{
    thread::spawn(move || {
        loop {
            match r.try_recv() {
                Ok(Message::Abort) => break,
                Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => {}
            }

            // get next job
            match q.next_job() {
                Job { mv, board, depth } => {
                    let val = board.alpha_beta(depth);
                    debug!("thread {} move={} val={}", threadnum, mv, val);
                    s.send((mv, val)).unwrap();
                }
            }
        }
    })
}

impl Threadpool {
    pub fn new(nthreads: usize) -> Threadpool {
        let mut hs = Vec::new();
        let mut cs = Vec::new();
        let (tx1, rx1) = channel();
        let q = Arc::new(JobQueue::new());

        for i in 0..nthreads {
            let (tx2, rx2) = channel();
            cs.push(tx2);
            hs.push(worker(i, rx2, tx1.clone(), q.clone()));
        }

        Threadpool {
            handles: hs,
            sends: cs,
            receive: rx1,
            jobs: q,
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

    pub fn find_best_move(&mut self, b: &Board) -> Result<(Move, isize), ChessError> {
        // notify the caller somehow when it is found
        let moves = b.legal_moves()?;
        for mv in moves.iter() {
            self.jobs.add_job(Job { mv: *mv, board: b.make_move(mv).unwrap(), depth: 5 });
        }

        let mut rng = rand::thread_rng();
        let mut best_score = isize::min_value();
        let mut best_move = None;
        for _ in 0..moves.len() {
            let (mv, score) = self.receive.recv().unwrap();
            if score > best_score || (score == best_score && rng.gen()) {
                best_move = Some(mv);
                best_score = score;
            }
        }

        Ok((best_move.unwrap(), best_score))
    }

    pub fn found_move(&mut self) -> Option<Move> {
        unimplemented!()
    }
}
