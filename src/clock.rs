use std::fmt;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub struct Clock {
    init: isize,
    time_left: Arc<Mutex<isize>>,
    running: Arc<Mutex<bool>>,
}

fn spawn_updater_thread(
    time_left: Arc<Mutex<isize>>,
    running: Arc<Mutex<bool>>,
    main_signal: Arc<Condvar>,
) {
    thread::spawn(move || loop {
        if !*running.lock().unwrap() {
            thread::sleep(Duration::from_millis(10));
            continue;
        }
        if *time_left.lock().unwrap() <= 0 {
            main_signal.notify_all();
        }
        let sleep_start = Instant::now();
        thread::sleep(Duration::from_millis(10));
        let slept_for = Instant::now().duration_since(sleep_start);
        *time_left.lock().unwrap() -= slept_for.subsec_nanos() as isize / 10000000;
    });
}

impl Clock {
    // starts stopped
    pub fn new(init: isize, main_signal: Arc<Condvar>) -> Clock {
        let time_left = Arc::new(Mutex::new(init));
        let running = Arc::new(Mutex::new(false));
        spawn_updater_thread(time_left.clone(), running.clone(), main_signal);
        Clock {
            init: init,
            time_left: time_left,
            running: running,
        }
    }

    pub fn reset(&self) {
        self.stop();
        *self.time_left.lock().unwrap() = self.init;
    }

    pub fn set(&mut self, init: isize) {
        self.init = init;
        *self.time_left.lock().unwrap() = init;
    }

    pub fn correct(&mut self, to: isize) {
        *self.time_left.lock().unwrap() = to;
    }

    pub fn is_zero(&self) -> bool {
        *self.time_left.lock().unwrap() <= 0
    }

    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }

    pub fn start(&self) {
        *self.running.lock().unwrap() = true;
    }

    pub fn time_remaining(&self) -> isize {
        *self.time_left.lock().unwrap()
    }
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let csecs = self.time_remaining();
        let secs = csecs / 100;
        if secs <= 0 {
            let min = -secs / 60;
            write!(f, "-{:01}:{:02}", min, (-secs) - min * 60)
        } else {
            let min = secs / 60;
            write!(f, "{:01}:{:02}", min, secs - min * 60)
        }
    }
}
