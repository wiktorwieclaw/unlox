use slab::Slab;

/// Generic tree data structure implementation.
pub struct Tree<T> {
    nodes: Slab<Node<T>>,
}

struct Node<T> {
    data: T,
    parent: Option<Index>,
    first_child: Option<Index>,
    last_child: Option<Index>,
    prev_sibling: Option<Index>,
    next_sibling: Option<Index>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Index(usize);

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
        }
    }
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn add_root(&mut self, value: T) -> Index {
        assert!(self.is_empty());
        Index(self.nodes.insert(Node {
            data: value,
            parent: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,
        }))
    }

    pub fn add_leaf(&mut self, parent: Index, value: T) -> Index {
        let parent_node = &self.nodes[parent.0];
        let prev_sibling = parent_node.last_child;

        let idx = Index(self.nodes.insert(Node {
            data: value,
            parent: Some(parent),
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling,
        }));

        let parent_node = &mut self.nodes[parent.0];
        parent_node.first_child.get_or_insert(idx);
        parent_node.last_child = Some(idx);
        if let Some(prev_sibling) = prev_sibling {
            self.nodes[prev_sibling.0].next_sibling = Some(idx);
        }
        idx
    }

    pub fn remove_leaf(&mut self, idx: Index) -> Option<T> {
        assert!(
            self.nodes
                .get(idx.0)
                .map_or(true, |n| n.first_child.is_none()),
            "Node is not a leaf"
        );
        let node = self.nodes.try_remove(idx.as_usize())?;
        if let Some(parent) = node.parent {
            let parent = &mut self.nodes[parent.0];

            if parent.first_child.is_some_and(|i| i == idx) {
                parent.first_child = node.next_sibling;
            }
            if parent.last_child.is_some_and(|i| i == idx) {
                parent.last_child = node.prev_sibling;
            }

            if let Some(prev_sibling) = node.prev_sibling {
                self.nodes[prev_sibling.0].next_sibling = node.next_sibling;
            }
            if let Some(next_sibling) = node.next_sibling {
                self.nodes[next_sibling.0].prev_sibling = node.prev_sibling;
            }
        }
        Some(node.data)
    }

    pub fn parent(&self, idx: Index) -> Option<Index> {
        self.nodes[idx.0].parent
    }

    pub fn node_data(&self, idx: Index) -> Option<&T> {
        self.nodes.get(idx.0).map(|n| &n.data)
    }

    pub fn node_data_mut(&mut self, idx: Index) -> Option<&mut T> {
        self.nodes.get_mut(idx.0).map(|n| &mut n.data)
    }
}

impl Index {
    pub fn as_usize(self) -> usize {
        self.0
    }
}
