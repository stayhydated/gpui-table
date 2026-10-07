//! Ordering application and selection snapshots for generated table delegates.

use gpui_kit::Context;
use gpui_kit::component::table::{TableDelegate, TableSelection, TableState};
use gpui_table_core::sort::{SortError, SortOrder};

/// Ordering of the delegate's loaded rows. Backend ordering remains caller-owned.
pub trait OrderedTableDelegate: TableDelegate {
    type RowId: Clone + Eq;
    fn ordering(&self) -> &SortOrder;
    fn set_ordering(&mut self, ordering: SortOrder) -> Result<(), SortError>;
    fn visible_row_id(&self, row_ix: usize) -> Option<Self::RowId>;
    fn visible_row_position(&self, id: &Self::RowId) -> Option<usize>;
}

/// Capture before changing rows or ordering, then restore by stable row identity.
pub struct TableRowSelection<Id> {
    row: Option<Id>,
    selection: TableSelection,
}

impl<Id: Clone + Eq> TableRowSelection<Id> {
    pub fn capture<D: OrderedTableDelegate<RowId = Id>>(table: &TableState<D>) -> Self {
        let selection = table.selection();
        let row_ix = match selection {
            TableSelection::Row(ix) | TableSelection::Cell(ix, _) => Some(ix),
            _ => None,
        };
        Self {
            row: row_ix.and_then(|ix| table.delegate().visible_row_id(ix)),
            selection,
        }
    }

    pub fn restore<D: OrderedTableDelegate<RowId = Id>>(
        self,
        table: &mut TableState<D>,
        cx: &mut Context<TableState<D>>,
    ) {
        let selection = match (self.row, self.selection) {
            (Some(id), TableSelection::Row(_)) => table
                .delegate()
                .visible_row_position(&id)
                .map(TableSelection::Row)
                .unwrap_or_default(),
            (Some(id), TableSelection::Cell(_, col)) => table
                .delegate()
                .visible_row_position(&id)
                .map(|ix| TableSelection::Cell(ix, col))
                .unwrap_or_default(),
            (_, selection) => selection,
        };
        table.set_selection(selection, cx);
    }
}

/// Apply controlled ordering while keeping row/cell selection on the same record.
/// This sorts loaded rows only; send the same order to a backend for complete-result ordering.
pub fn set_table_ordering<D: OrderedTableDelegate>(
    table: &mut TableState<D>,
    ordering: SortOrder,
    cx: &mut Context<TableState<D>>,
) -> Result<(), SortError> {
    let selected = TableRowSelection::capture(table);
    table.delegate_mut().set_ordering(ordering)?;
    table.refresh(cx);
    selected.restore(table, cx);
    cx.notify();
    Ok(())
}
