#![allow(unused)]

use arc_swap::ArcSwap;
use dashmap::DashSet;
use std::cell::Cell;
use std::hash::Hash;
use std::sync::{Arc, Mutex, RwLock};
use thread_local::ThreadLocal;

pub trait Strategy: Send + Sync {
    fn next_index(&self, size: usize) -> usize;
}

pub struct RoundRobin(ThreadLocal<Cell<usize>>);

impl Default for RoundRobin {
    fn default() -> Self {
        Self(ThreadLocal::new())
    }
}

impl Strategy for RoundRobin {
    fn next_index(&self, size: usize) -> usize {
        let cell = self.0.get_or(|| Cell::new(0));
        let val = cell.get();
        cell.set(val.wrapping_add(1));
        val % size
    }
}

#[derive(Default)]
pub struct Random;

impl Strategy for Random {
    fn next_index(&self, size: usize) -> usize {
        use rand::RngExt;
        rand::rngs::ThreadRng::default().random_range(0..size)
    }
}

pub struct Balancer<T> {
    nodes: ArcSwap<Vec<T>>,
    strategy: RwLock<Box<dyn Strategy>>,
    set: DashSet<T>,
    write_lock: Mutex<()>,
}

unsafe impl<T: Send + Sync> Send for Balancer<T> {}
unsafe impl<T: Send + Sync> Sync for Balancer<T> {}

impl<T> Balancer<T>
where
    T: Clone + Eq + Hash + Send + Sync + 'static,
{
    pub fn new<S: Strategy + 'static>(strategy: S) -> Self {
        Balancer {
            nodes: ArcSwap::from(Arc::new(Vec::new())),
            strategy: RwLock::new(Box::new(strategy)),
            set: DashSet::new(),
            write_lock: Mutex::new(()),
        }
    }

    pub fn add(&self, node: T) {
        if self.set.insert(node.clone()) {
            let _guard = self.write_lock.lock().unwrap();
            let mut new_nodes = (**self.nodes.load()).clone();
            new_nodes.push(node);
            self.nodes.store(Arc::new(new_nodes));
        }
    }

    pub fn remove(&self, node: T) -> bool {
        if self.set.remove(&node).is_some() {
            let _guard = self.write_lock.lock().unwrap();
            let mut new_nodes = (**self.nodes.load()).clone();
            new_nodes.retain(|n| *n != node);
            self.nodes.store(Arc::new(new_nodes));
            return true;
        }
        false
    }

    pub fn next(&self) -> Option<T> {
        let nodes = self.nodes.load();
        if nodes.is_empty() {
            return None;
        }
        let idx = self.strategy.read().unwrap().next_index(nodes.len());
        Some(nodes[idx].clone())
    }

    pub fn set_strategy<S: Strategy + 'static>(&self, strategy: S) {
        *self.strategy.write().unwrap() = Box::new(strategy);
    }
}

fn main() {
    //
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_round_robin_sequential() {
        let balancer = Balancer::new(RoundRobin::default());
        balancer.add(1);
        balancer.add(2);
        balancer.add(3);

        assert_eq!(balancer.next(), Some(1));
        assert_eq!(balancer.next(), Some(2));
        assert_eq!(balancer.next(), Some(3));
        assert_eq!(balancer.next(), Some(1));
    }

    #[test]
    fn test_random_generates_output() {
        let balancer = Balancer::new(Random);
        balancer.add(1);
        balancer.add(2);
        balancer.add(3);

        let result = balancer.next().unwrap();
        assert!(result == 1 || result == 2 || result == 3);
    }

    #[test]
    fn test_remove() {
        let balancer = Balancer::new(RoundRobin::default());
        balancer.add(10);
        balancer.add(20);
        balancer.add(30);

        balancer.remove(20);
        assert_eq!(balancer.next(), Some(10));
        assert_eq!(balancer.next(), Some(30));
        assert_eq!(balancer.next(), Some(10));
    }

    #[test]
    fn test_empty() {
        let balancer: Balancer<i32> = Balancer::new(RoundRobin::default());
        assert_eq!(balancer.next(), None);
    }

    #[test]
    fn test_add_idempotent() {
        let balancer = Balancer::new(RoundRobin::default());
        balancer.add(5);
        balancer.add(5);

        assert_eq!(balancer.next(), Some(5));
        assert_eq!(balancer.next(), Some(5));
    }

    #[test]
    fn test_set_strategy() {
        let balancer = Balancer::new(RoundRobin::default());
        balancer.add(1);
        balancer.add(2);

        balancer.set_strategy(Random);
        let result = balancer.next().unwrap();
        assert!(result == 1 || result == 2);
    }

    #[test]
    fn test_many_nodes() {
        let balancer = Balancer::new(RoundRobin::default());
        for i in 1..=100 {
            balancer.add(i);
        }
        for i in 1..=100 {
            assert_eq!(balancer.next(), Some(i));
        }
    }

    #[test]
    fn test_thread_safety() {
        let balancer = Arc::new(Balancer::new(RoundRobin::default()));
        let mut handles = vec![];

        for i in 0..4 {
            let b = balancer.clone();
            handles.push(std::thread::spawn(move || {
                b.add(i);
                b.add(i + 10);
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        for _ in 0..20 {
            assert!(balancer.next().is_some());
        }
    }

    #[test]
    fn test_concurrent_next() {
        let balancer = Arc::new(Balancer::new(RoundRobin::default()));
        for i in 0..10 {
            balancer.add(i);
        }

        let mut handles = vec![];
        for _ in 0..5 {
            let b = balancer.clone();
            handles.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    let _ = b.next();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }
    }
}
