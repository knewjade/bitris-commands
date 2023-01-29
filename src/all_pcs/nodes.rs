use bitris::placements::PlacedPiece;
use derive_more::Constructor;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug, Constructor)]
pub(crate) struct IndexId {
    pub id: usize,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug, Constructor)]
pub(crate) struct ItemId {
    pub id: usize,
}

pub(crate) enum IndexNode {
    // Indicates the placement of one of the items.
    // 0(usize) is the head of the item id, and 1(usize) has the count of items (next_item_id, item_length).
    ToItem(ItemId, u32),

    // Indicates no placement.
    // 0(usize) is the index id to be seen next (next_index_id).
    ToNextIndex(IndexId),

    // Indicates that the sequence is valid.
    Complete,

    // Indicates that the sequence is invalid.
    Abort,
}

pub(crate) struct ItemNode {
    pub(crate) placed_piece: PlacedPiece,
    pub(crate) next_index_id: IndexId,
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

    pub(crate) fn next_index_id(&self) -> IndexId {
        IndexId::new(self.indexes.len())
    }

    pub(crate) fn next_item_id(&self) -> ItemId {
        ItemId::new(self.items.len())
    }

    pub(crate) fn go_to_items(&mut self, next_item_id: ItemId, item_length: u32) {
        debug_assert!(0 < item_length);
        self.indexes.push(IndexNode::ToItem(next_item_id, item_length));
    }

    pub(crate) fn skip_to_next_index(&mut self, next_index_id: IndexId) {
        self.indexes.push(IndexNode::ToNextIndex(next_index_id));
    }

    pub(crate) fn complete(&mut self) {
        self.indexes.push(IndexNode::Complete);
    }

    pub(crate) fn abort(&mut self) {
        self.indexes.push(IndexNode::Abort);
    }

    pub(crate) fn push_item(&mut self, placed_piece: PlacedPiece, next_index_id: IndexId) {
        self.items.push(ItemNode { placed_piece, next_index_id });
    }

    #[inline]
    pub(crate) fn head_index_id(&self) -> Option<IndexId> {
        if self.indexes.is_empty() {
            None
        } else {
            Some(IndexId::new(0))
        }
    }

    #[inline]
    pub(crate) fn index(&self, id: IndexId) -> Option<&IndexNode> {
        self.indexes.get(id.id)
    }

    #[inline]
    pub(crate) fn item(&self, id: ItemId) -> Option<&ItemNode> {
        self.items.get(id.id)
    }
}
