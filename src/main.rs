use std::{
    net::{IpAddr, SocketAddr},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use dashmap::DashMap;

// Задача: написать библиотеку балансировщика нагрузки с одним алгоритмом балансировки на выбор и возможностью расширения новыми алгоритмами. Должны быть поддержаны три основные функции: добавил узел в список для балансирования, удалить узел и получить следующий узел по алгоритму балансировки.
//
// Дополнительные требования:
// 1. Методы должны быть асинхронными.
// 2. Библиотека должна быть потокобезопасной - один объект балансировщика может использоваться несколькими потокам.
// 3. Балансировщик должен быть производительным и выдерживать большую нагрузку.
// 4. Должна быть возможность менять стратегию балансировки в рантайме.
// 5. Модель узла не имеет значения, достаточно идентификатора

type Key = u64;

#[derive(Default, Debug)]
struct RoundRobinStrategy {
    last_key: Option<Key>,
}

#[derive(Default, Debug)]
struct RandomStrategy;

#[derive(Debug)]
enum Strategy {
    RoundRobin(RoundRobinStrategy),
    Random(RandomStrategy),
}

#[derive(Debug)]
struct Node {
    key: u64,
    addr: SocketAddr,
}

#[derive(Debug)]
struct Balancer {
    key_generator: AtomicU64,
    strategy: Mutex<Strategy>,
    nodes: DashMap<Key, Node>,
}

impl Balancer {
    fn new(strategy: Strategy) -> Self {
        Self {
            key_generator: AtomicU64::new(0),
            strategy: Mutex::new(strategy),
            nodes: Default::default(),
        }
    }

    fn gen_key(&self) -> u64 {
        self.key_generator.fetch_add(1, Ordering::SeqCst)
    }

    fn set_stragety(&self, strategy: Strategy) {
        //
    }

    fn add_node(&self, node: Node) {
        if let Some(prev) = self.nodes.insert(node.key, node) {
            println!("Node override: {}", prev.key);
        }
    }

    fn pick_node(&self) -> Node {
        todo!()
    }
}

fn main() {
    let strategy = Strategy::RoundRobin(RoundRobinStrategy::default());
    let balancer = Balancer::new(strategy);
    let balancer = Arc::new(balancer);

    let new_id = balancer.gen_key();
    let node = Node {
        key: new_id,
        addr: "127.0.0.1:1234".parse().unwrap(),
    };
    balancer.add_node(node);

    println!("Hello, world!");
}
