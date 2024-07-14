use slab::Slab;

/// Generic "Cactus stack" data structure, also known as "Parent pointer tree".
pub struct Cactus<T> {
    nodes: Slab<Node<T>>,
    current: Option<Index>,
}

pub struct Node<T> {
    data: T,
    parent: Option<Index>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Index(usize);

impl<T> Default for Cactus<T> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            current: None,
        }
    }
}

impl<T> Cactus<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn push(&mut self, value: T) -> Index {
        let idx = Index(self.nodes.insert(Node {
            data: value,
            parent: self.current,
        }));
        self.current = Some(idx);
        idx
    }

    pub fn pop(&mut self) -> Option<T> {
        let node = self.nodes.try_remove(self.current?.as_usize())?;
        self.current = node.parent;
        Some(node.data)
    }

    pub fn current(&self) -> Option<Index> {
        self.current
    }

    pub fn current_node(&self) -> Option<&Node<T>> {
        self.node(self.current?)
    }

    pub fn current_node_mut(&mut self) -> Option<&mut Node<T>> {
        self.nodes.get_mut(self.current?.as_usize())
    }

    pub fn node(&self, idx: Index) -> Option<&Node<T>> {
        self.nodes.get(idx.as_usize())
    }

    pub fn node_data(&self, idx: Index) -> Option<&T> {
        self.node(idx).map(|node| &node.data)
    }

    pub fn node_data_mut(&mut self, idx: Index) -> Option<&mut T> {
        self.nodes
            .get_mut(idx.as_usize())
            .map(|node| &mut node.data)
    }
}

impl<T> Node<T> {
    pub fn parent(&self) -> Option<Index> {
        self.parent
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl Index {
    pub fn as_usize(&self) -> usize {
        self.0
    }
}
