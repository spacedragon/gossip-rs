use std::collections::HashMap;
use rand::{thread_rng, seq};

pub trait Versioned {
    fn version(&self) -> i64;
}

enum NodeState {
    on,
    off,
}

pub struct RawNode<T> {
    state: NodeState,
    seed: bool,
    raw: T,
}

pub struct Diff<V> {
    needs: Vec<String>,
    changes: Vec<(String, V)>,
}

pub struct GossipErr {}

pub trait GossipHandler<T, V: Versioned + Clone> {
    fn select_nodes<'a>(&self, nodes: Vec<&'a RawNode<T>>) -> Result<Vec<&'a RawNode<T>>, GossipErr> {
        let mut rng = thread_rng();
        let r = seq::sample_iter(&mut rng, nodes, 1);
        return r.map_err(|_| GossipErr {});
    }

    fn syn(&self, nodes: Vec<&RawNode<T>>, versions: HashMap<String, i64>) -> Result<Diff<V>, GossipErr>;
    fn ack(&self, updates: Vec<(String, &V)>) -> Result<(), GossipErr>;
}

struct Options {}

pub struct Gossip<N, V: Versioned + Clone> {
    data: HashMap<String, V>,
    nodes: Vec<RawNode<N>>,
    local_node: N,
}

impl<N: PartialEq, V: Versioned + Clone> Gossip<N, V> {
    pub fn new(local_node: N, seeds: Vec<N>) -> Gossip<N, V> {
        let nodes = seeds.into_iter()
            .map(|r| {
                RawNode {
                    state: NodeState::on,
                    seed: true,
                    raw: r,
                }
            }).collect();

        return Gossip {
            data: HashMap::new(),
            nodes,
            local_node,
        };
    }


    pub fn gossip(&mut self, handler: &GossipHandler<N, V>) -> Result<(), GossipErr> {
        let remote_nodes =
            self.nodes.iter().filter(|n| n.raw.ne(&self.local_node)).collect();

        let selected = handler.select_nodes(remote_nodes)?;
        let versions = self.data.iter().map(|(k, v)| {
            (k.clone(), v.version())
        }).collect();
        let diff = handler.syn(selected, versions)?;
        for (k, v) in diff.changes {
            let new_ver = v.version();
            if self.data.contains_key(&k) {
                if let Some(old) = self.data.get_mut(&k) {
                    if old.version() < new_ver {
                        *old = v;
                    }
                }
            } else {
                self.data.insert(k, v);
            }
        }
        let updates = diff.needs.into_iter().flat_map(
            |n| {
                self.data.get(&n).map(|v| (n, v)).into_iter()
            }
        ).collect();
        handler.ack(updates)
    }

    fn receiving_syn(&mut self, mut versions: HashMap<String, i64>) -> Diff<V> {
        let mut diff: Diff<V> = Diff {
            needs: Vec::new(),
            changes: Vec::new(),
        };
        for (k, v) in self.data.iter() {
            let my_ver = v.version();
            if let Some(&that_ver) = versions.get(k) {
                if that_ver > my_ver {
                    diff.needs.push(k.clone());
                } else if that_ver < my_ver {
                    diff.changes.push((k.clone(), v.clone()));
                }
            } else {
                diff.changes.push((k.clone(), v.clone()));
            }
            versions.remove(k);
        }
        for (k, _) in versions {
            diff.needs.push(k)
        }
        return diff;
    }
}
