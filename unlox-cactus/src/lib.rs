use slab::Slab;

/// Generic "Cactus stack" data structure, also known as "Parent pointer tree".
#[derive(Debug, Clone)]
pub struct Cactus<T> {
    nodes: Slab<Node<T>>,
    current: Option<Index>,
}

// Ensure `Node` remains private to protect tree's invariants.
#[derive(Debug, Clone)]
struct Node<T> {
    data: T,
    parent: Option<Index>,
}

/// Node index.
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

    pub fn contains(&self, idx: Index) -> bool {
        self.nodes.contains(idx.0)
    }

    /// Pushes node on top of the active stack frame.
    pub fn push(&mut self, value: T) -> Index {
        let idx = Index(self.nodes.insert(Node {
            data: value,
            parent: self.current,
        }));
        self.current = Some(idx);
        idx
    }

    /// Pops node out of the active stack frame.
    pub fn pop(&mut self) -> Option<T> {
        let node = self.nodes.try_remove(self.current?.as_usize())?;
        self.current = node.parent;
        Some(node.data)
    }

    /// Returns the index of the top node of the active stack frame.
    pub fn current(&self) -> Option<Index> {
        self.current
    }

    /// Returns index of the parent's node.
    ///
    /// # Panics if node doesn't exist
    pub fn parent(&self, idx: Index) -> Option<Index> {
        self.nodes[idx.as_usize()].parent
    }

    /// Returns a reference to node's data by given `idx`.
    pub fn node_data(&self, idx: Index) -> Option<&T> {
        self.nodes.get(idx.as_usize()).map(|n| &n.data)
    }

    /// Returns a mutable reference to node's data by given `idx`.
    pub fn node_data_mut(&mut self, idx: Index) -> Option<&mut T> {
        self.nodes.get_mut(idx.as_usize()).map(|n| &mut n.data)
    }
}

impl Index {
    pub fn as_usize(&self) -> usize {
        self.0
    }
}
