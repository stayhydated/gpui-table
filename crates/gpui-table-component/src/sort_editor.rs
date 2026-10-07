//! Controlled sort-clause editing without owning a query, dialog or persistence.

use es_fluent::EsFluent;
use gpui_kit::component::{
    Disableable, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    menu::{DropdownMenu as _, PopupMenuItem},
    v_flex,
};
use gpui_kit::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    StyleRefinement, Styled, Window, div,
};
use gpui_table_core::sort::{
    MAX_SORT_CLAUSES, NullPlacement, SortClause, SortDirection, SortError, SortOrder,
};
use std::rc::Rc;

#[derive(Clone, Copy, EsFluent)]
enum SortEditorFtl {
    AddKey,
    Ascending,
    Descending,
    NullsFirst,
    NullsLast,
    MoveUp,
    MoveDown,
    Remove,
    Empty,
}

/// A sortable key and its caller-localized title.
#[derive(Clone, Debug)]
pub struct SortColumnOption {
    key: String,
    title: String,
}
impl SortColumnOption {
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
        }
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn title(&self) -> &str {
        &self.title
    }
}

type ChangeHandler = Rc<dyn Fn(SortOrder, &mut Window, &mut App)>;

/// Controlled ordered-key controls. Owners apply requested changes and rerender.
/// Column keys namespace row controls, so moving a clause preserves its identity.
#[derive(IntoElement)]
pub struct SortEditor {
    id: ElementId,
    columns: Vec<SortColumnOption>,
    ordering: SortOrder,
    disabled: bool,
    on_change: ChangeHandler,
    style: StyleRefinement,
}
impl SortEditor {
    pub fn new(
        id: impl Into<ElementId>,
        columns: Vec<SortColumnOption>,
        ordering: SortOrder,
        on_change: impl Fn(SortOrder, &mut Window, &mut App) + 'static,
    ) -> Result<Self, SortError> {
        let mut keys = std::collections::BTreeSet::new();
        for column in &columns {
            if column.key.is_empty() || column.key.trim() != column.key {
                return Err(SortError::InvalidColumn(column.key.clone()));
            }
            if !keys.insert(&column.key) {
                return Err(SortError::DuplicateColumn(column.key.clone()));
            }
        }
        for clause in ordering.clauses() {
            if !columns.iter().any(|column| column.key == clause.column()) {
                return Err(SortError::UnsupportedColumn(clause.column().into()));
            }
        }
        Ok(Self {
            id: id.into(),
            columns,
            ordering,
            disabled: false,
            on_change: Rc::new(on_change),
            style: StyleRefinement::default(),
        })
    }
    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }
}
impl Disableable for SortEditor {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
impl Styled for SortEditor {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

fn emit(clauses: Vec<SortClause>, handler: &ChangeHandler, window: &mut Window, cx: &mut App) {
    // Actions only edit validated existing keys or insert a validated unused key.
    handler(
        SortOrder::new(clauses).expect("validated editor clauses"),
        window,
        cx,
    );
}
impl RenderOnce for SortEditor {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let label = |message| crate::i18n::localize_message(cx, &message);
        let mut content = v_flex().id(self.id).gap_2().refine_style(&self.style);
        if self.ordering.is_empty() {
            content = content.child(div().text_sm().child(label(SortEditorFtl::Empty)));
        }
        for (ix, clause) in self.ordering.clauses().iter().enumerate() {
            let key = clause.column().to_owned();
            let title = self
                .columns
                .iter()
                .find(|column| column.key == key)
                .expect("validated editor key")
                .title
                .clone();
            let options = self
                .columns
                .iter()
                .filter(|column| {
                    column.key == key
                        || !self
                            .ordering
                            .clauses()
                            .iter()
                            .any(|clause| clause.column() == column.key)
                })
                .cloned()
                .collect::<Vec<_>>();
            let selected_key = key.clone();
            let clauses = self.ordering.clauses().to_vec();
            let on_change = self.on_change.clone();
            let column = Button::new(format!("sort-column-{key}"))
                .outline()
                .small()
                .label(title)
                .disabled(self.disabled)
                .dropdown_menu(move |mut menu, _, _| {
                    for option in &options {
                        let mut next = clauses.clone();
                        next[ix] = SortClause::new(&option.key, next[ix].direction())
                            .with_nulls(next[ix].nulls());
                        let on_change = on_change.clone();
                        menu = menu.item(
                            PopupMenuItem::new(option.title.clone())
                                .checked(option.key == selected_key)
                                .on_click(move |_, window, cx| {
                                    emit(next.clone(), &on_change, window, cx)
                                }),
                        );
                    }
                    menu
                });
            let direction_label = label(match clause.direction() {
                SortDirection::Ascending => SortEditorFtl::Ascending,
                SortDirection::Descending => SortEditorFtl::Descending,
            });
            let mut next = self.ordering.clauses().to_vec();
            let direction = match clause.direction() {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
            next[ix] = SortClause::new(&key, direction).with_nulls(clause.nulls());
            let on_change = self.on_change.clone();
            let direction = Button::new(format!("sort-direction-{key}"))
                .outline()
                .small()
                .label(direction_label)
                .disabled(self.disabled)
                .on_click(move |_, window, cx| emit(next.clone(), &on_change, window, cx));
            let null_label = label(match clause.nulls() {
                NullPlacement::First => SortEditorFtl::NullsFirst,
                NullPlacement::Last => SortEditorFtl::NullsLast,
            });
            let mut next = self.ordering.clauses().to_vec();
            let nulls = match clause.nulls() {
                NullPlacement::First => NullPlacement::Last,
                NullPlacement::Last => NullPlacement::First,
            };
            next[ix] = SortClause::new(&key, clause.direction()).with_nulls(nulls);
            let on_change = self.on_change.clone();
            let nulls = Button::new(format!("sort-nulls-{key}"))
                .outline()
                .small()
                .label(null_label)
                .disabled(self.disabled)
                .on_click(move |_, window, cx| emit(next.clone(), &on_change, window, cx));
            let mut row = h_flex()
                .id(format!("sort-clause-{key}"))
                .gap_2()
                .child(column)
                .child(direction)
                .child(nulls);
            for (verb, destination, disabled, message) in [
                ("up", ix.saturating_sub(1), ix == 0, SortEditorFtl::MoveUp),
                (
                    "down",
                    ix + 1,
                    ix + 1 == self.ordering.clauses().len(),
                    SortEditorFtl::MoveDown,
                ),
            ] {
                let mut next = self.ordering.clauses().to_vec();
                if !disabled {
                    next.swap(ix, destination);
                }
                let on_change = self.on_change.clone();
                row = row.child(
                    Button::new(format!("sort-{verb}-{key}"))
                        .ghost()
                        .small()
                        .label(label(message))
                        .disabled(self.disabled || disabled)
                        .on_click(move |_, window, cx| emit(next.clone(), &on_change, window, cx)),
                );
            }
            let mut next = self.ordering.clauses().to_vec();
            next.remove(ix);
            let on_change = self.on_change.clone();
            row = row.child(
                Button::new(format!("sort-remove-{key}"))
                    .ghost()
                    .small()
                    .label(label(SortEditorFtl::Remove))
                    .disabled(self.disabled)
                    .on_click(move |_, window, cx| emit(next.clone(), &on_change, window, cx)),
            );
            content = content.child(row);
        }
        let available = self
            .columns
            .iter()
            .filter(|column| {
                !self
                    .ordering
                    .clauses()
                    .iter()
                    .any(|clause| clause.column() == column.key)
            })
            .cloned()
            .collect::<Vec<_>>();
        let disabled = self.disabled
            || available.is_empty()
            || self.ordering.clauses().len() == MAX_SORT_CLAUSES;
        let clauses = self.ordering.clauses().to_vec();
        let on_change = self.on_change;
        content.child(
            Button::new("sort-add-key")
                .outline()
                .small()
                .label(label(SortEditorFtl::AddKey))
                .disabled(disabled)
                .dropdown_menu(move |mut menu, _, _| {
                    for option in &available {
                        let mut next = clauses.clone();
                        next.push(SortClause::new(&option.key, SortDirection::Ascending));
                        let on_change = on_change.clone();
                        menu = menu.item(PopupMenuItem::new(option.title.clone()).on_click(
                            move |_, window, cx| emit(next.clone(), &on_change, window, cx),
                        ));
                    }
                    menu
                }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::component::Root;
    use gpui_kit::test::TestWindowExt;
    use gpui_kit::{AppContext as _, Context, Entity, Render, TestAppContext, px, size};

    struct Harness {
        ordering: SortOrder,
        changes: usize,
        disabled: bool,
    }
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let owner = cx.entity();
            SortEditor::new(
                "sort-editor",
                vec![
                    SortColumnOption::new("category", "Category"),
                    SortColumnOption::new("score", "Score"),
                    SortColumnOption::new("age", "Age"),
                ],
                self.ordering.clone(),
                move |ordering, _, cx| {
                    owner.update(cx, |owner, cx| {
                        owner.ordering = ordering;
                        owner.changes += 1;
                        cx.notify();
                    });
                },
            )
            .unwrap()
            .disabled(self.disabled)
        }
    }

    #[gpui_kit::test]
    fn native_controls_edit_priority_direction_and_keys_once_per_action(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        cx.update(|cx| crate::i18n::init(cx).unwrap());
        let mut owner: Option<Entity<Harness>> = None;
        let window = cx.open_window(size(px(960.), px(480.)), |window, cx| {
            let harness = cx.new(|_| Harness {
                ordering: SortOrder::new(vec![
                    SortClause::new("score", SortDirection::Ascending),
                    SortClause::new("category", SortDirection::Ascending),
                ])
                .unwrap(),
                changes: 0,
                disabled: false,
            });
            owner = Some(harness.clone());
            Root::new(harness, window, cx)
        });
        let owner = owner.unwrap();
        cx.update_window(window.into(), |_, window, cx| {
            window.render_frame(cx);
            window.click("sort-direction-score", cx);
            assert_eq!(
                owner.read(cx).ordering.clauses()[0].direction(),
                SortDirection::Descending
            );
            assert_eq!(owner.read(cx).changes, 1);
            // The window host establishes initial focus before keyboard traversal.
            window.focus_next(cx);
            window.render_frame(cx);
            for _ in 0..16 {
                if window.find("sort-direction-score").focused() == Some(true) {
                    break;
                }
                window.press("tab", cx);
            }
            assert_eq!(window.find("sort-direction-score").focused(), Some(true));
            window.press("enter", cx);
            assert_eq!(
                owner.read(cx).ordering.clauses()[0].direction(),
                SortDirection::Ascending
            );
            assert_eq!(owner.read(cx).changes, 2);
            window.click("sort-down-score", cx);
            assert_eq!(owner.read(cx).ordering.clauses()[0].column(), "category");
            assert_eq!(owner.read(cx).changes, 3);
            window.click("sort-nulls-score", cx);
            assert_eq!(
                owner.read(cx).ordering.clauses()[1].nulls(),
                NullPlacement::First
            );
            assert_eq!(owner.read(cx).changes, 4);
            window.click("sort-remove-score", cx);
            assert_eq!(owner.read(cx).ordering.clauses().len(), 1);
            window.click("sort-add-key", cx);
            window.press("down", cx);
            window.press("enter", cx);
            assert_eq!(owner.read(cx).ordering.clauses()[1].column(), "score");
            assert_eq!(owner.read(cx).changes, 6);
            owner.update(cx, |owner, cx| {
                owner.disabled = true;
                cx.notify();
            });
            window.render_frame(cx);
            let unchanged = owner.read(cx).ordering.clone();
            window.click("sort-direction-score", cx);
            window.press("enter", cx);
            assert_eq!(owner.read(cx).changes, 6);
            assert_eq!(owner.read(cx).ordering, unchanged);
        })
        .unwrap();
    }
}
