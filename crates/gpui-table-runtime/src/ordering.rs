//! Ordering application and selection snapshots for generated table delegates.

use gpui_kit::component::table::{Column, TableDelegate, TableSelection, TableState};
use gpui_kit::{AnyElement, App, Context, Window};
use gpui_table_core::sort::{ResolvedSortColumns, SortError, SortOrder, SortableRow};

/// Ordering of the delegate's loaded rows. Backend ordering remains caller-owned.
pub trait OrderedTableDelegate: TableDelegate {
    type RowId: Clone + Eq;
    fn ordering(&self) -> &SortOrder;
    fn set_ordering(&mut self, ordering: SortOrder) -> Result<(), SortError>;
    fn visible_row_id(&self, row_ix: usize) -> Option<Self::RowId>;
    fn visible_row_position(&self, id: &Self::RowId) -> Option<usize>;
}

/// A source-owned executable resolved-column context for the delegate's typed rows.
pub trait ResolvedOrderedTableDelegate<R: SortableRow>: OrderedTableDelegate {
    /// Current executable additions to native row keys.
    fn resolved_sort_columns(&self) -> &ResolvedSortColumns<R>;
    /// Atomically validate and replace the retained comparison context.
    fn set_resolved_sort_columns(
        &mut self,
        columns: ResolvedSortColumns<R>,
    ) -> Result<(), SortError>;
}

type ResolvedCell<R> = dyn Fn(&R, &mut Window, &mut App) -> AnyElement;

/// Presentation for an executable resolved key, rendered from the current typed row.
/// The caller keeps formatting separate from the comparison extractor.
pub struct ResolvedTableColumn<R> {
    column: Column,
    render: std::rc::Rc<ResolvedCell<R>>,
}
impl<R> Clone for ResolvedTableColumn<R> {
    fn clone(&self) -> Self {
        Self {
            column: self.column.clone(),
            render: self.render.clone(),
        }
    }
}
impl<R> ResolvedTableColumn<R> {
    pub fn new(
        column: Column,
        render: impl Fn(&R, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Result<Self, SortError> {
        SortOrder::new(vec![gpui_table_core::sort::SortClause::new(
            column.key.to_string(),
            gpui_table_core::sort::SortDirection::Ascending,
        )])?;
        Ok(Self {
            column,
            render: std::rc::Rc::new(render),
        })
    }
    pub fn column(&self) -> &Column {
        &self.column
    }
    pub fn render(&self, row: &R, window: &mut Window, cx: &mut App) -> AnyElement {
        (self.render)(row, window, cx)
    }
}

/// Controlled presentation columns attached to the same resolved-key context.
pub trait ResolvedTableDelegate<R: SortableRow>: ResolvedOrderedTableDelegate<R> {
    fn resolved_table_columns(&self) -> &[ResolvedTableColumn<R>];
    fn set_resolved_table_columns(
        &mut self,
        columns: Vec<ResolvedTableColumn<R>>,
    ) -> Result<(), SortError>;
}

/// Refresh presentation while retaining the selected record's stable identity.
pub fn set_table_resolved_columns<R: SortableRow, D: ResolvedTableDelegate<R>>(
    table: &mut TableState<D>,
    columns: Vec<ResolvedTableColumn<R>>,
    cx: &mut Context<TableState<D>>,
) -> Result<(), SortError> {
    let mut selected = TableRowSelection::capture(table);
    let column_key = match selected.selection {
        TableSelection::Cell(_, column) => Some(table.delegate().column(column, cx).key),
        _ => None,
    };
    table.delegate_mut().set_resolved_table_columns(columns)?;
    if let (Some(key), TableSelection::Cell(row, _)) = (column_key, selected.selection) {
        selected.selection = (0..table.delegate().columns_count(cx))
            .find(|column| table.delegate().column(*column, cx).key == key)
            .map_or(TableSelection::Row(row), |column| {
                TableSelection::Cell(row, column)
            });
    }
    table.refresh(cx);
    selected.restore(table, cx);
    cx.notify();
    Ok(())
}

/// Capture before changing rows or ordering, then restore by stable row identity.
#[derive(Clone)]
pub struct TableRowSelection<Id> {
    row: Option<Id>,
    selection: TableSelection,
}

impl<Id: Clone + Eq> TableRowSelection<Id> {
    /// Selected row identity, available to callers retaining a paged selection.
    pub fn row_id(&self) -> Option<&Id> {
        self.row.as_ref()
    }

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

/// Replace the resolved comparison context while retaining selection by stable identity.
pub fn set_table_resolved_sort_columns<R: SortableRow, D: ResolvedOrderedTableDelegate<R>>(
    table: &mut TableState<D>,
    columns: ResolvedSortColumns<R>,
    cx: &mut Context<TableState<D>>,
) -> Result<(), SortError> {
    let selected = TableRowSelection::capture(table);
    table.delegate_mut().set_resolved_sort_columns(columns)?;
    table.refresh(cx);
    selected.restore(table, cx);
    cx.notify();
    Ok(())
}
