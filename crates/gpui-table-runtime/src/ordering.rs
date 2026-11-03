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

/// Capture before changing rows or ordering, then restore all selected stable identities.
#[derive(Clone)]
pub struct TableRowSelection<Id> {
    row: Option<Id>,
    rows: Vec<Id>,
    #[cfg(feature = "multi-row-selection")]
    cursor: Option<Id>,
    #[cfg(feature = "multi-row-selection")]
    anchor: Option<Id>,
    #[cfg(feature = "multi-row-selection")]
    revision: u64,
    selection: TableSelection,
}

impl<Id: Clone + Eq> TableRowSelection<Id> {
    /// Current selected cursor identity (or selected cell), retained for compatibility.
    pub fn row_id(&self) -> Option<&Id> {
        self.row.as_ref()
    }

    /// Exact selected row identities, including a selected cell's record.
    pub fn row_ids(&self) -> &[Id] {
        &self.rows
    }

    /// The explicit input revision captured before a source refresh.
    pub fn selection_revision(&self) -> u64 {
        #[cfg(feature = "multi-row-selection")]
        {
            self.revision
        }
        #[cfg(not(feature = "multi-row-selection"))]
        {
            0
        }
    }

    pub fn all_rows_visible<D: OrderedTableDelegate<RowId = Id>>(&self, delegate: &D) -> bool {
        self.rows
            .iter()
            .all(|id| delegate.visible_row_position(id).is_some())
    }

    /// Additive input retains previously selected records outside the loaded view.
    pub fn preserve_unavailable<D: OrderedTableDelegate<RowId = Id>>(
        &mut self,
        old: &Self,
        delegate: &D,
    ) {
        for id in &old.rows {
            if delegate.visible_row_position(id).is_none() && !self.rows.contains(id) {
                self.rows.push(id.clone());
            }
        }
        if !self.rows.is_empty() && self.selection == TableSelection::None {
            self.selection = TableSelection::Row(0);
        }
    }

    /// Authoritative deletion removes only that identity, including a parked record.
    pub fn remove_row_id(&mut self, id: &Id) {
        self.rows.retain(|row| row != id);
        if self.row.as_ref() == Some(id) {
            self.row = self.rows.first().cloned();
        }
        #[cfg(feature = "multi-row-selection")]
        {
            if self.cursor.as_ref() == Some(id) {
                self.cursor = None;
            }
            if self.anchor.as_ref() == Some(id) {
                self.anchor = None;
            }
        }
        if self.rows.is_empty()
            && matches!(
                self.selection,
                TableSelection::Row(_) | TableSelection::Cell(_, _)
            )
        {
            self.selection = TableSelection::None;
        }
    }

    pub fn capture<D: OrderedTableDelegate<RowId = Id>>(table: &TableState<D>) -> Self {
        let selection = table.selection();
        let row_ix = match selection {
            TableSelection::Row(ix) | TableSelection::Cell(ix, _) => Some(ix),
            _ => None,
        };
        let row = row_ix.and_then(|ix| table.delegate().visible_row_id(ix));
        #[cfg(not(feature = "multi-row-selection"))]
        let rows = row.iter().cloned().collect();
        #[cfg(feature = "multi-row-selection")]
        let rows = if matches!(selection, TableSelection::Cell(_, _)) {
            row.iter().cloned().collect()
        } else {
            table
                .selected_rows()
                .iter()
                .filter_map(|ix| table.delegate().visible_row_id(*ix))
                .collect()
        };
        Self {
            row,
            rows,
            #[cfg(feature = "multi-row-selection")]
            cursor: table
                .row_selection_cursor()
                .and_then(|ix| table.delegate().visible_row_id(ix)),
            #[cfg(feature = "multi-row-selection")]
            anchor: table
                .row_selection_anchor()
                .and_then(|ix| table.delegate().visible_row_id(ix)),
            #[cfg(feature = "multi-row-selection")]
            revision: table.selection_revision(),
            selection,
        }
    }

    pub fn restore<D: OrderedTableDelegate<RowId = Id>>(
        self,
        table: &mut TableState<D>,
        cx: &mut Context<TableState<D>>,
    ) {
        #[cfg(feature = "multi-row-selection")]
        if matches!(self.selection, TableSelection::Row(_)) {
            let rows = self
                .rows
                .iter()
                .filter_map(|id| table.delegate().visible_row_position(id))
                .collect();
            let cursor = self
                .cursor
                .and_then(|id| table.delegate().visible_row_position(&id));
            let anchor = self
                .anchor
                .and_then(|id| table.delegate().visible_row_position(&id));
            table.restore_row_selection(rows, cursor, anchor, cx);
            return;
        }
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
        #[cfg(feature = "multi-row-selection")]
        table.restore_selection(selection, cx);
        #[cfg(not(feature = "multi-row-selection"))]
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
