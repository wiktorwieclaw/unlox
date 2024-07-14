use slab::Slab;

/// Generic tree data structure implementation.
pub struct Tree<T> {
    nodes: Slab<Node<T>>,
}

pub struct Node<T> {
    value: T,
    parent: Option<Index>,
    children: Vec<Index>,
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
        Index(self.nodes.insert(Node::root(value)))
    }

    pub fn add_leaf(&mut self, parent: Index, value: T) -> Index {
        assert!(self.get(parent).is_some());
        let idx = Index(self.nodes.insert(Node::leaf(value, parent)));
        self.get_mut(parent).unwrap().children.push(idx);
        idx
    }

    pub fn remove_leaf(&mut self, idx: Index) -> Option<T> {
        assert!(
            self.get(idx).map_or(true, |n| n.children.is_empty()),
            "Node is not a leaf"
        );
        let node = self.nodes.try_remove(idx.as_usize())?;
        if let Some(parent) = node.parent {
            let parent = self.get_mut(parent).unwrap();
            let pos = parent.children.iter().position(|i| *i == idx).unwrap();
            parent.children.remove(pos);
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

impl<T> Node<T> {
    fn root(value: T) -> Self {
        Self {
            value,
            parent: None,
            children: Vec::new(),
        }
    }

    fn leaf(value: T, parent: Index) -> Self {
        Self {
            value,
            parent: Some(parent),
            children: Vec::new(),
        }
    }

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
