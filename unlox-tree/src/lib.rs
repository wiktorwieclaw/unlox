/// Generic tree data structure implementation.
#[derive(Default)]
pub struct Tree<T> {
    nodes: Vec<Option<Node<T>>>,
}

pub struct Node<T> {
    value: T,
    parent: Option<Index>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Index(usize);

impl<T> Tree<T> {
    pub fn add_root(&mut self, value: T) -> Index {
        assert!(self.is_empty());
        self.add(Node::root(value))
    }

    pub fn add_leaf(&mut self, parent: Index, value: T) -> Index {
        assert!(self.get(parent).is_some());
        self.add(Node::leaf(value, parent))
    }

    fn add(&mut self, node: Node<T>) -> Index {
        let idx = self.nodes.iter_mut().position(|slot| slot.is_none());
        if let Some(idx) = idx {
            self.nodes[idx] = Some(node);
            Index(idx)
        } else {
            let idx = self.nodes.len();
            self.nodes.push(Some(node));
            Index(idx)
        }
    }

    pub fn get(&self, idx: Index) -> Option<&Node<T>> {
        self.nodes.get(idx.0)?.as_ref()
    }

    pub fn get_mut(&mut self, idx: Index) -> Option<&mut Node<T>> {
        self.nodes.get_mut(idx.0)?.as_mut()
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
        }
    }

    fn leaf(value: T, parent: Index) -> Self {
        Self {
            value,
            parent: Some(parent),
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
