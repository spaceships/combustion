use board::Board;

use std::collections::HashMap;
use std::sync::RwLock;

pub struct TranspositionTable {
    depth_tables: Vec<RwLock<HashMap<String, isize>>>,
}

impl TranspositionTable {
    pub fn new(max_depth: usize) -> TranspositionTable {
        let mut tabs = Vec::with_capacity(max_depth);
        for _ in 0..max_depth {
            tabs.push(RwLock::new(HashMap::new()));
        }
        TranspositionTable {
            depth_tables: tabs,
        }
    }

    pub fn get(&self, b: &Board, depth: usize) -> Option<isize> {
        let ref map = self.depth_tables[depth].read().unwrap();
        map.get(&b.to_fen()).map(|res| res.clone())
    }

    pub fn insert(&self, b: &Board, depth: usize, result: isize) {
        let ref tab = self.depth_tables[depth];
        let ref mut map = tab.write().unwrap();
        map.insert(b.to_fen(), result);
    }
}
