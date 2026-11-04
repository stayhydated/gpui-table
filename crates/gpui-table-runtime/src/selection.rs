//! Delegate-owned multiple selection composed with the public GPUI Kit table.
use crate::OrderedTableDelegate;
use gpui_kit::component::{
    ActiveTheme as _,
    table::{TableSelection, TableState},
};
use gpui_kit::{
    App, Context, Div, Entity, Focusable as _, InteractiveElement as _, Modifiers, Stateful,
    StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _,
};

#[derive(Clone, Default)]
pub struct RowSelectionState {
    enabled: bool,
    rows: Vec<usize>,
    cursor: Option<usize>,
    anchor: Option<usize>,
    revision: u64,
    preserve_unavailable: bool,
}

pub fn new_multi_row_table<D: OrderedTableDelegate>(
    mut delegate: D,
    window: &mut Window,
    cx: &mut Context<TableState<D>>,
) -> TableState<D> {
    if let Some(selection) = delegate.row_selection_mut() {
        selection.enabled = true;
    }
    TableState::new(delegate, window, cx).select_on_modifier_click(false)
}

pub fn enable_row_selection<D: OrderedTableDelegate>(table: &mut TableState<D>) {
    if let Some(state) = table.delegate_mut().row_selection_mut() {
        state.enabled = true;
    }
}

pub fn selected_rows<D: OrderedTableDelegate>(table: &TableState<D>) -> Vec<usize> {
    if let Some(state) = table
        .delegate()
        .row_selection()
        .filter(|state| state.enabled)
    {
        return state.rows.clone();
    }
    table.selected_row().into_iter().collect()
}
pub fn selection_revision<D: OrderedTableDelegate>(table: &TableState<D>) -> u64 {
    table
        .delegate()
        .row_selection()
        .map_or(0, |state| state.revision)
}
pub fn preserves_unavailable_rows<D: OrderedTableDelegate>(table: &TableState<D>) -> bool {
    table
        .delegate()
        .row_selection()
        .is_some_and(|state| state.preserve_unavailable)
}
pub fn row_selection_cursor<D: OrderedTableDelegate>(table: &TableState<D>) -> Option<usize> {
    table
        .delegate()
        .row_selection()
        .filter(|state| state.enabled)
        .and_then(|state| state.cursor)
        .or(table.selected_row())
}
pub fn row_selection_anchor<D: OrderedTableDelegate>(table: &TableState<D>) -> Option<usize> {
    table
        .delegate()
        .row_selection()
        .filter(|state| state.enabled)
        .and_then(|state| state.anchor)
        .or(table.selected_row())
}

pub fn restore_rows<D: OrderedTableDelegate>(
    table: &mut TableState<D>,
    mut rows: Vec<usize>,
    cursor: Option<usize>,
    anchor: Option<usize>,
    cx: &mut Context<TableState<D>>,
) {
    let count = table.delegate().rows_count(cx);
    rows.retain(|row| *row < count);
    rows.sort_unstable();
    rows.dedup();
    let cursor = cursor
        .filter(|row| *row < count)
        .or_else(|| rows.first().copied());
    let primary = cursor
        .filter(|row| rows.contains(row))
        .or_else(|| rows.first().copied());
    if let Some(state) = table.delegate_mut().row_selection_mut() {
        state.rows = rows;
        state.cursor = cursor;
        state.anchor = anchor.filter(|row| *row < count).or(cursor);
    }
    match primary {
        Some(row) => table.set_selected_row(row, cx),
        None => table.clear_selection(cx),
    }
    cx.notify();
}

pub fn select_row<D: OrderedTableDelegate>(
    table: &mut TableState<D>,
    row: usize,
    modifiers: Modifiers,
    cx: &mut Context<TableState<D>>,
) {
    if !table.row_selectable || row >= table.delegate().rows_count(cx) {
        return;
    }
    let enabled = table
        .delegate()
        .row_selection()
        .is_some_and(|state| state.enabled);
    let additive = enabled && modifiers.secondary(cx);
    let anchor = if enabled && modifiers.shift {
        row_selection_anchor(table)
            .or(row_selection_cursor(table))
            .unwrap_or(row)
    } else {
        row
    };
    let mut rows = if additive {
        selected_rows(table)
    } else {
        Vec::new()
    };
    if enabled && modifiers.shift {
        rows.extend(anchor.min(row)..=anchor.max(row));
    } else if additive && rows.contains(&row) {
        rows.retain(|selected| *selected != row);
    } else {
        rows.push(row);
    }
    if let Some(state) = table.delegate_mut().row_selection_mut() {
        state.revision = state.revision.wrapping_add(1);
        state.preserve_unavailable = additive;
    }
    restore_rows(table, rows, Some(row), Some(anchor), cx);
}

pub fn clear_selection<D: OrderedTableDelegate>(
    table: &mut TableState<D>,
    cx: &mut Context<TableState<D>>,
) {
    if let Some(state) = table.delegate_mut().row_selection_mut() {
        state.revision = state.revision.wrapping_add(1);
        state.preserve_unavailable = false;
    }
    restore_rows(table, Vec::new(), None, None, cx);
}

pub fn decorate_row<D: OrderedTableDelegate>(
    row: Stateful<Div>,
    index: usize,
    selection: &RowSelectionState,
    window: &mut Window,
    cx: &mut Context<TableState<D>>,
) -> Stateful<Div> {
    if !selection.enabled {
        return row;
    }
    let selected = selection.rows.contains(&index);
    row.when(selected, |row| row.bg(cx.theme().table_active))
        .on_click(window.listener_for(
            &cx.entity(),
            move |table, event: &gpui_kit::ClickEvent, window, cx| {
                table.focus_handle(cx).focus(window, cx);
                select_row(table, index, event.modifiers(), cx);
                cx.stop_propagation();
            },
        ))
}

/// Wrap tables or alternate card/grid presentations sharing the delegate.
pub fn selection_container<D: OrderedTableDelegate>(
    table: &Entity<TableState<D>>,
    window: &mut Window,
    cx: &App,
) -> Stateful<Div> {
    selection_scope(table, window, cx)
        .key_context("DataTable")
        .track_focus(&table.focus_handle(cx))
}

/// Captures selection commands above an existing native table focus owner.
pub fn selection_scope<D: OrderedTableDelegate>(
    table: &Entity<TableState<D>>,
    window: &mut Window,
    cx: &App,
) -> Stateful<Div> {
    let focus = table.focus_handle(cx);
    div()
        .id("table-selection")
        .capture_action(window.listener_for(
            table,
            |table, _: &gpui_kit::base::actions::Cancel, window, cx| {
                if !handle_navigation(table, "escape", window.modifiers(), window, cx) {
                    cx.propagate();
                }
            },
        ))
        .capture_action(window.listener_for(
            table,
            |table, _: &gpui_kit::base::actions::SelectUp, window, cx| {
                if !handle_navigation(table, "up", window.modifiers(), window, cx) {
                    cx.propagate();
                }
            },
        ))
        .capture_action(window.listener_for(
            table,
            |table, _: &gpui_kit::base::actions::SelectDown, window, cx| {
                if !handle_navigation(table, "down", window.modifiers(), window, cx) {
                    cx.propagate();
                }
            },
        ))
        .capture_action(window.listener_for(
            table,
            |table, _: &gpui_kit::base::actions::SelectFirst, window, cx| {
                if !handle_navigation(table, "home", window.modifiers(), window, cx) {
                    cx.propagate();
                }
            },
        ))
        .capture_action(window.listener_for(
            table,
            |table, _: &gpui_kit::base::actions::SelectLast, window, cx| {
                if !handle_navigation(table, "end", window.modifiers(), window, cx) {
                    cx.propagate();
                }
            },
        ))
        .capture_action(window.listener_for(
            table,
            |table, _: &gpui_kit::base::actions::SelectPageUp, window, cx| {
                if !handle_navigation(table, "pageup", window.modifiers(), window, cx) {
                    cx.propagate();
                }
            },
        ))
        .capture_action(window.listener_for(
            table,
            |table, _: &gpui_kit::base::actions::SelectPageDown, window, cx| {
                if !handle_navigation(table, "pagedown", window.modifiers(), window, cx) {
                    cx.propagate();
                }
            },
        ))
        .capture_key_down(window.listener_for(
            table,
            move |table, event: &gpui_kit::KeyDownEvent, window, cx| {
                if focus.is_focused(window) {
                    handle_navigation(
                        table,
                        &event.keystroke.key,
                        event.keystroke.modifiers,
                        window,
                        cx,
                    );
                }
            },
        ))
}

fn handle_navigation<D: OrderedTableDelegate>(
    table: &mut TableState<D>,
    key: &str,
    modifiers: Modifiers,
    window: &mut Window,
    cx: &mut Context<TableState<D>>,
) -> bool {
    if !table.focus_handle(cx).is_focused(window)
        || !table
            .delegate()
            .row_selection()
            .is_some_and(|state| state.enabled)
        || matches!(
            table.selection(),
            TableSelection::Cell(..) | TableSelection::Column(..)
        )
    {
        return false;
    }
    let count = table.delegate().rows_count(cx);
    if key == "escape" {
        clear_selection(table, cx);
    } else if key == "a" && modifiers.secondary(cx) {
        if let Some(state) = table.delegate_mut().row_selection_mut() {
            state.revision = state.revision.wrapping_add(1);
            state.preserve_unavailable = false;
        }
        restore_rows(
            table,
            (0..count).collect(),
            row_selection_cursor(table).or((count > 0).then_some(0)),
            row_selection_anchor(table),
            cx,
        );
    } else if count > 0 {
        let cursor = row_selection_cursor(table);
        let row = match key {
            "down" => cursor.map_or(0, |row| (row + 1).min(count - 1)),
            "up" => cursor.map_or(count - 1, |row| row.saturating_sub(1)),
            "home" => 0,
            "end" => count - 1,
            "pageup" => cursor.unwrap_or(0).saturating_sub(10),
            "pagedown" => cursor.unwrap_or(0).saturating_add(10).min(count - 1),
            _ => return false,
        };
        select_row(table, row, modifiers, cx);
    } else {
        return false;
    }
    cx.stop_propagation();
    true
}
