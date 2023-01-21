pub(crate) enum IndexNode {
    // (next_item_id, item_length)
    Jump(u32, u32),

    // (next_index_id)
    Skip(u32),

    ToHi,
}

pub(crate) enum ItemNode {
    // (mino_index)
    ToHi(u32),

    // (mino_index, next_index_id)
    ToIndex(u32, usize),
}

impl ItemNode {
    pub(crate) fn mino_index(&self) -> u32 {
        return *match self {
            ItemNode::ToHi(mino_index) => { mino_index }
            ItemNode::ToIndex(mino_index, _) => { mino_index }
        };
    }
}

pub(crate) struct Nodes {
    pub(crate) indexes: Vec<IndexNode>,
    pub(crate) items: Vec<ItemNode>,
}

impl Nodes {
    pub(crate) fn empty() -> Self {
        Self { indexes: Vec::new(), items: Vec::new() }
    }

    pub(crate) fn index_serial(&self) -> usize {
        self.indexes.len()
    }

    pub(crate) fn item_serial(&self) -> usize {
        self.items.len()
    }

    pub(crate) fn jump(&mut self, next_item_id: u32, item_length: u32) {
        self.indexes.push(IndexNode::Jump(next_item_id, item_length));
    }

    pub(crate) fn skip(&mut self, next_index_id: u32) {
        self.indexes.push(IndexNode::Skip(next_index_id));
    }

    pub(crate) fn complete2(&mut self) {
        self.indexes.push(IndexNode::ToHi);
    }

    pub(crate) fn put(&mut self, mino_index: u32, next_index_id: usize) {
        self.items.push(ItemNode::ToIndex(mino_index, next_index_id));
    }

    pub(crate) fn complete(&mut self, mino_index: u32) {
        self.items.push(ItemNode::ToHi(mino_index));
    }
}
