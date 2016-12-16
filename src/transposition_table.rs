use board::Board;

use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::Mutex;

pub struct TranspositionTable {
    depth_tables: Vec<Mutex<RefCell<HashMap<String, isize>>>>,
}

impl TranspositionTable {
    pub fn new(max_depth: usize) -> TranspositionTable {
        let mut tabs = Vec::with_capacity(max_depth);
        for _ in 0..max_depth {
            tabs.push(Mutex::new(RefCell::new(HashMap::new())));
        }
        TranspositionTable {
            depth_tables: tabs,
        }
    }

    pub fn get(&self, b: &Board, depth: usize) -> Option<isize> {
        let ref cell = *self.depth_tables[depth].lock().unwrap();
        let map = cell.borrow();
        map.get(&b.to_fen()).map(|res| res.clone())
    }

    pub fn insert(&self, b: &Board, depth: usize, result: isize) {
        let ref cell = *self.depth_tables[depth].lock().unwrap();
        let mut map = cell.borrow_mut();
        map.insert(b.to_fen(), result);
    }
}
