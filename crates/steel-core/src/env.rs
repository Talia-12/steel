#[cfg(feature = "sync")]
use parking_lot::{RwLock, RwLockReadGuard};
#[cfg(feature = "sync")]
use std::sync::Arc;

use crate::rvals::{Result, SteelVal};

#[allow(unused)]
#[derive(Debug)]
pub struct Env {
    #[cfg(not(feature = "sync"))]
    pub(crate) bindings_vec: Vec<SteelVal>,

    // Globals from one thread, need to be able to refer to
    // globals from another thread. So in order to do so,
    // there needs to be a lock on all globals since that way
    // things are relatively consistent.
    #[cfg(feature = "sync")]
    pub(crate) bindings_vec: Arc<RwLock<Vec<SteelVal>>>,
    // Keep a copy of the globals that we can access
    // just by offset.
    // #[cfg(feature = "sync")]
    // pub(crate) thread_local_bindings: Vec<SteelVal>,
}

#[cfg(feature = "sync")]
impl Clone for Env {
    fn clone(&self) -> Self {
        Self {
            bindings_vec: self.bindings_vec.clone(),
            // thread_local_bindings: self.thread_local_bindings.clone(),
        }
    }
}

#[cfg(not(feature = "sync"))]
impl Clone for Env {
    fn clone(&self) -> Self {
        Self {
            bindings_vec: self.bindings_vec.clone(),
        }
    }
}

#[cfg(not(feature = "sync"))]
impl Env {
    pub fn extract(&self, idx: usize) -> Option<SteelVal> {
        self.bindings_vec.get(idx).cloned()
    }

    pub fn len(&self) -> usize {
        self.bindings_vec.len()
    }

    /// top level global env has no parent
    pub fn root() -> Self {
        Env {
            bindings_vec: Vec::with_capacity(1024),
        }
    }

    #[cfg(feature = "dynamic")]
    pub(crate) fn _print_diagnostics(&self) {
        for (idx, value) in self.bindings_vec.iter().enumerate() {
            if let SteelVal::Closure(b) = value {
                let count = b.call_count();
                if count > 0 {
                    println!("Function: {} - Count: {}", idx, b.call_count());
                }
            }
        }
    }

    #[inline(always)]
    pub fn repl_lookup_idx(&self, idx: usize) -> SteelVal {
        self.bindings_vec[idx].clone()
    }

    /// Get the value located at that index
    pub fn _repl_get_idx(&self, idx: usize) -> &SteelVal {
        &self.bindings_vec[idx]
    }

    #[inline]
    pub fn repl_define_idx(&mut self, idx: usize, val: SteelVal) {
        if idx < self.bindings_vec.len() {
            self.bindings_vec[idx] = val;
        } else {
            if idx > self.bindings_vec.len() {
                // TODO: This seems suspect. Try to understand
                // what is happening here. This would be that values
                // are getting interned to be at a global offset in the
                // wrong order, which seems to be fine in general,
                // assuming that the values then get actually updated
                // to the correct values.
                for _ in 0..(idx - self.bindings_vec.len()) {
                    self.bindings_vec.push(SteelVal::Void);
                }
            }

            self.bindings_vec.push(val);
            assert_eq!(self.bindings_vec.len() - 1, idx);
        }
    }

    pub fn repl_set_idx(&mut self, idx: usize, val: SteelVal) -> Result<SteelVal> {
        let output = self.bindings_vec[idx].clone();
        self.bindings_vec[idx] = val;
        Ok(output)
    }

    #[inline]
    pub fn add_root_value(&mut self, idx: usize, val: SteelVal) {
        // self.bindings_map.insert(idx, val);
        self.repl_define_idx(idx, val);
    }

    pub fn roots(&self) -> &Vec<SteelVal> {
        &self.bindings_vec
    }
}

#[cfg(feature = "sync")]
impl Env {
    pub fn extract(&self, idx: usize) -> Option<SteelVal> {
        self.bindings_vec.read().get(idx).cloned()
    }

    pub fn len(&self) -> usize {
        self.bindings_vec.read().len()
    }

    /// top level global env has no parent
    pub fn root() -> Self {
        Env {
            bindings_vec: Arc::new(RwLock::new(Vec::with_capacity(1024))),
            // thread_local_bindings: Vec::with_capacity(1024),
        }
    }

    pub fn deep_clone(&self) -> Self {
        Self {
            bindings_vec: Arc::new(RwLock::new(
                self.bindings_vec.read().iter().map(|x| x.clone()).collect(),
            )),
            // thread_local_bindings: self.thread_local_bindings.clone(),
        }
    }

    #[cfg(feature = "dynamic")]
    pub(crate) fn _print_diagnostics(&self) {
        for (idx, value) in self.bindings_vec.iter().enumerate() {
            if let SteelVal::Closure(b) = value {
                let count = b.call_count();
                if count > 0 {
                    println!("Function: {} - Count: {}", idx, b.call_count());
                }
            }
        }
    }

    #[inline(always)]
    pub fn repl_lookup_idx(&self, idx: usize) -> SteelVal {
        self.bindings_vec.read()[idx].clone()
        // self.thread_local_bindings[idx].clone()
    }

    // /// Get the value located at that index
    // pub fn _repl_get_idx(&self, idx: usize) -> &SteelVal {
    //     &self.bindings_vec.read()[idx]
    // }

    #[inline]
    pub fn repl_define_idx(&mut self, idx: usize, val: SteelVal) {
        let mut guard = self.bindings_vec.write();

        if idx < guard.len() {
            guard[idx] = val.clone();
            // self.thread_local_bindings[idx] = val;
        } else {
            if idx > guard.len() {
                // TODO: This seems suspect. Try to understand
                // what is happening here. This would be that values
                // are getting interned to be at a global offset in the
                // wrong order, which seems to be fine in general,
                // assuming that the values then get actually updated
                // to the correct values.
                for _ in 0..(idx - guard.len()) {
                    guard.push(SteelVal::Void);
                    // self.thread_local_bindings.push(SteelVal::Void);
                }
            }

            guard.push(val.clone());
            // self.thread_local_bindings.push(val);
            assert_eq!(guard.len() - 1, idx);
        }
    }

    pub fn repl_set_idx(&mut self, idx: usize, val: SteelVal) -> Result<SteelVal> {
        let mut guard = self.bindings_vec.write();
        let output = guard[idx].clone();
        guard[idx] = val.clone();
        // self.thread_local_bindings[idx] = val;
        Ok(output)
    }

    #[inline]
    pub fn add_root_value(&mut self, idx: usize, val: SteelVal) {
        // self.bindings_map.insert(idx, val);
        self.repl_define_idx(idx, val);
    }

    // TODO: This needs to be fixed!
    #[cfg(feature = "sync")]
    pub fn roots(&self) -> RwLockReadGuard<'_, Vec<SteelVal>> {
        self.bindings_vec.read()
    }

    #[cfg(not(feature = "sync"))]
    pub fn roots(&self) -> &Vec<SteelVal> {
        &self.bindings_vec
    }
}
