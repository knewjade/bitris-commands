pub(crate) enum IndexNode {
    // Indicates the placement of one of the items.
    // 0(usize) is the head of the item id, and 1(usize) has the count of items (next_item_id, item_length).
    ToItem(usize, u32),

    // Indicates no placement.
    // 0(usize) is the index id to be seen next (next_index_id).
    ToNextIndex(usize),

    // Indicates that the sequence is valid.
    Complete,

    // Indicates that the sequence is invalid.
    Abort,
}

pub(crate) struct ItemNode {
    pub(crate) predefine_index: u32,
    pub(crate) next_index_id: usize,
}

// Starting at `indexes[0]` and moving back and forth between `indexes` and `items`, it represents a sequence of placements.
// `IndexNode::ToItem` means to place one piece and jump to the next index once it is placed.
// Follow until `Complete` or `Abort` appears, and the sequence can be `valid` or `invalid` when it appears.
pub(crate) struct Nodes {
    pub(crate) indexes: Vec<IndexNode>,
    pub(crate) items: Vec<ItemNode>,
}

impl Nodes {
    pub(crate) fn empty() -> Self {
        Self { indexes: Vec::new(), items: Vec::new() }
    }

    pub(crate) fn next_index_id(&self) -> usize {
        self.indexes.len()
    }

    pub(crate) fn next_item_id(&self) -> usize {
        self.items.len()
    }

    pub(crate) fn go_to_items(&mut self, next_item_id: usize, item_length: u32) {
        debug_assert!(0 < item_length);
        self.indexes.push(IndexNode::ToItem(next_item_id, item_length));
    }

    pub(crate) fn skip_to_next_index(&mut self, next_index_id: usize) {
        self.indexes.push(IndexNode::ToNextIndex(next_index_id));
    }

    pub(crate) fn complete(&mut self) {
        self.indexes.push(IndexNode::Complete);
    }

    pub(crate) fn abort(&mut self) {
        self.indexes.push(IndexNode::Abort);
    }

    pub(crate) fn push_item(&mut self, predefine_index: u32, next_index_id: usize) {
        self.items.push(ItemNode { predefine_index, next_index_id });
    }

    pub(crate) fn head(&self) -> Option<&IndexNode> {
        self.indexes.get(0)
    }
}
