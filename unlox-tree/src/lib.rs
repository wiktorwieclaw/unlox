use slab::Slab;

/// Generic tree data structure implementation.
pub struct Tree<T> {
    nodes: Slab<Node<T>>,
}

pub struct Node<T> {
    value: T,
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

    pub fn add_root(&mut self, value: T) -> Index {
        assert!(self.is_empty());
        Index(self.nodes.insert(Node {
            value,
            parent: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,
        }))
    }

    pub fn add_leaf(&mut self, parent: Index, value: T) -> Index {
        let parent_node = &self[parent];
        let prev_sibling = parent_node.last_child;

        let idx = Index(self.nodes.insert(Node {
            value,
            parent: Some(parent),
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling,
        }));

        let parent_node = &mut self[parent];
        parent_node.first_child.get_or_insert(idx);
        parent_node.last_child = Some(idx);
        if let Some(prev_sibling) = prev_sibling {
            self[prev_sibling].next_sibling = Some(idx);
        }
        idx
    }

    pub fn remove_leaf(&mut self, idx: Index) -> Option<T> {
        assert!(
            self.get(idx).map_or(true, |n| n.first_child.is_none()),
            "Node is not a leaf"
        );
        let node = self.nodes.try_remove(idx.as_usize())?;
        if let Some(parent) = node.parent {
            let parent = &mut self[parent];

            if parent.first_child.is_some_and(|i| i == idx) {
                parent.first_child = node.next_sibling;
            }
            if parent.last_child.is_some_and(|i| i == idx) {
                parent.last_child = node.prev_sibling;
            }

            if let Some(prev_sibling) = node.prev_sibling {
                self[prev_sibling].next_sibling = node.next_sibling;
            }
            if let Some(next_sibling) = node.next_sibling {
                self[next_sibling].prev_sibling = node.prev_sibling;
            }
        }
        Some(node.value)
    }

    pub fn get(&self, idx: Index) -> Option<&Node<T>> {
        self.nodes.get(idx.0)
    }

    pub fn get_mut(&mut self, idx: Index) -> Option<&mut Node<T>> {
        self.nodes.get_mut(idx.0)
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl<T> std::ops::Index<Index> for Tree<T> {
    type Output = Node<T>;

    fn index(&self, index: Index) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T> std::ops::IndexMut<Index> for Tree<T> {
    fn index_mut(&mut self, index: Index) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<T> Node<T> {
    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub fn parent(&self) -> Option<Index> {
        self.parent
    }
}

impl Index {
    pub fn as_usize(self) -> usize {
        self.0
    }
}
