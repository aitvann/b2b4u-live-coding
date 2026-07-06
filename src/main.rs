use std::sync::{Arc, Mutex};
use std::{fmt, net::SocketAddr};

// Задача: написать библиотеку балансировщика нагрузки с одним алгоритмом балансировки на выбор и возможностью расширения новыми алгоритмами. Должны быть поддержаны три основные функции: добавил узел в список для балансирования, удалить узел и получить следующий узел по алгоритму балансировки.
//
// Дополнительные требования:
// 1. Методы должны быть асинхронными.
// 2. Библиотека должна быть потокобезопасной - один объект балансировщика может использоваться несколькими потокам.
// 3. Балансировщик должен быть производительным и выдерживать большую нагрузку.
// 4. Должна быть возможность менять стратегию балансировки в рантайме.
// 5. Модель узла не имеет значения, достаточно идентификатора

pub trait Strategy: fmt::Debug {
    fn route<'a>(&mut self, nodes: &'a [Node]) -> &'a Node;
}

#[derive(Default, Debug)]
pub struct RoundRobin {
    current_node_idx: usize,
}

impl Strategy for RoundRobin {
    fn route<'a>(&mut self, nodes: &'a [Node]) -> &'a Node {
        if self.current_node_idx >= nodes.len() {
            self.current_node_idx = 0;
        }

        let node = &nodes[self.current_node_idx];
        self.current_node_idx += 1;
        node
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Node {
    addr: SocketAddr,
}

#[derive(Debug)]
pub struct Balancer {
    strategy: Mutex<Box<dyn Strategy>>,
    nodes: Mutex<Vec<Node>>,
}

impl Balancer {
    pub fn new(strategy: Box<dyn Strategy>) -> Self {
        Self {
            strategy: Mutex::new(strategy),
            nodes: Mutex::new(vec![]),
        }
    }

    pub fn add_node(&self, node: Node) {
        let mut guard = self.nodes.lock().unwrap();
        guard.push(node);
    }

    pub fn pick_node(&self) -> Node {
        let mut strategy = self.strategy.lock().unwrap();
        let nodes = self.nodes.lock().unwrap();
        let node = strategy.route(&nodes);
        node.clone()
    }

    #[allow(dead_code)]
    pub fn set_strategy(&self, strategy: Box<dyn Strategy>) {
        *self.strategy.lock().unwrap() = strategy;
    }
}

fn main() {
    let strategy = RoundRobin::default();
    let balancer = Balancer::new(Box::new(strategy));
    // let balancer = Arc::new(balancer);

    let node1 = Node {
        addr: "127.0.0.1:1234".parse().unwrap(),
    };
    balancer.add_node(node1);

    let node2 = Node {
        addr: "127.0.0.1:4567".parse().unwrap(),
    };
    balancer.add_node(node2);

    dbg!(balancer.pick_node());
    dbg!(balancer.pick_node());
    dbg!(balancer.pick_node());
    dbg!(balancer.pick_node());
    dbg!(balancer.pick_node());
    dbg!(balancer.pick_node());
    dbg!(balancer.pick_node());
    dbg!(balancer.pick_node());
}
