mod gossip;

extern crate rand;


#[cfg(test)]
mod tests {
    use gossip::*;
    use std::collections::HashMap;
    use std::sync::{Mutex, Arc};

    #[derive(PartialEq, Clone)]
    struct Node {
        address: &'static str,
    }

    #[derive(Debug, Clone)]
    struct Value {
        ver: i64
    }

    impl Versioned for Value {
        fn version(&self) -> i64 {
            self.ver
        }
    }


    struct LocalServer {
        local_node: Node,
        gossip: Gossip<Node, Value>,
    }

    impl LocalServer {
        fn new(local_node: Node, seeds: Vec<Node>) -> LocalServer {
            return LocalServer {
                local_node: local_node.clone(),
                gossip: Gossip::new(local_node.clone(), seeds),
            };
        }
        fn gossip(&mut self, handler: &GossipHandler<Node, Value>) {
            self.gossip.gossip(handler);
        }
    }

    struct Handler {
        servers: Vec<Arc<Mutex<LocalServer>>>
    }

    impl GossipHandler<Node, Value> for Handler {
        fn syn(&self, nodes: Vec<&RawNode<Node>>, versions: HashMap<String, i64>) -> Result<Diff<Value>, GossipErr> {
            println!("{:?}", versions);
            unimplemented!()
        }

        fn ack(&self, updates: Vec<(String, &Value)>) -> Result<(), GossipErr> {
            unimplemented!()
        }
    }

    #[test]
    fn it_works() {
        let node1 = Node { address: "1" };
        let node2 = Node { address: "2" };
        let node3 = Node { address: "3" };

        let seeds = vec![
            node1.clone(),
            node2.clone(),
            node3.clone()
        ];
        let server1 = Arc::new(Mutex::new(LocalServer::new(node1, seeds.clone())));
        let server2 = Arc::new(Mutex::new(LocalServer::new(node2, seeds.clone())));
        let server3 = Arc::new(Mutex::new(LocalServer::new(node3, seeds)));
        let servers = vec![server1.clone(), server2.clone(), server3.clone()];
        let handler = Handler {
            servers
        };
        {
            let mut s1 = server1.lock().unwrap();
            s1.gossip(&handler);
        }
    }
}
