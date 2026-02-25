use crate::buffer::Buffer;

#[derive(Clone)]
pub struct Snapshot {
    pub buffer: Buffer,
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub cursor_row_offset: u16,
    pub was_dirty: bool,
}

pub struct UndoHistory {
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
    capacity: usize,
}

impl UndoHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            capacity,
        }
    }

    /// 変更前に呼ぶ。redo スタックをクリアする。
    pub fn commit(&mut self, snapshot: Snapshot) {
        self.redo_stack.clear();
        self.undo_stack.push(snapshot);
        if self.undo_stack.len() > self.capacity {
            self.undo_stack.remove(0);
        }
    }

    /// 現在状態を渡して、直前スナップショットを返す。
    pub fn undo(&mut self, current: Snapshot) -> Option<Snapshot> {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(current);
            Some(prev)
        } else {
            None
        }
    }

    /// 現在状態を渡して、redo スナップショットを返す。
    pub fn redo(&mut self, current: Snapshot) -> Option<Snapshot> {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(current);
            Some(next)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;

    fn make_snapshot(dirty: bool) -> Snapshot {
        Snapshot {
            buffer: Buffer::new(),
            cursor_x: 1,
            cursor_y: 1,
            cursor_row_offset: 0,
            was_dirty: dirty,
        }
    }

    #[test]
    fn test_undo_empty() {
        let mut history = UndoHistory::new(100);
        let current = make_snapshot(false);
        assert!(history.undo(current).is_none());
    }

    #[test]
    fn test_undo_after_commit() {
        let mut history = UndoHistory::new(100);
        let snap = make_snapshot(false);
        history.commit(snap);

        let current = make_snapshot(true);
        let result = history.undo(current);
        assert!(result.is_some());
        assert!(!result.unwrap().was_dirty);

        // 再 undo は None
        let current2 = make_snapshot(false);
        assert!(history.undo(current2).is_none());
    }

    #[test]
    fn test_undo_then_redo() {
        let mut history = UndoHistory::new(100);
        history.commit(make_snapshot(false));

        let after_edit = make_snapshot(true);
        let prev = history.undo(after_edit).unwrap();
        assert!(!prev.was_dirty);

        let current = make_snapshot(false);
        let redone = history.redo(current).unwrap();
        assert!(redone.was_dirty);
    }

    #[test]
    fn test_new_commit_clears_redo() {
        let mut history = UndoHistory::new(100);
        history.commit(make_snapshot(false));

        // undo してから新しい操作
        let after = make_snapshot(true);
        history.undo(after).unwrap();
        // redo スタックに 1 件ある
        assert!(history.redo_stack.len() == 1);

        // 新しい commit で redo クリア
        history.commit(make_snapshot(false));
        assert!(history.redo_stack.is_empty());
    }

    #[test]
    fn test_capacity_eviction() {
        let mut history = UndoHistory::new(3);
        for i in 0..5 {
            history.commit(Snapshot {
                buffer: Buffer::new(),
                cursor_x: i as u16,
                cursor_y: 1,
                cursor_row_offset: 0,
                was_dirty: true,
            });
        }
        // capacity=3 なので 3 件しか残らない
        assert_eq!(history.undo_stack.len(), 3);
        // 最も古いものが削除されているはず
        assert_eq!(history.undo_stack[0].cursor_x, 2);
    }
}
