use crate::{SortColumnOption, SortEditor};
use gpui_kit::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Window,
};
use gpui_table_core::sort::{SortClause, SortDirection, SortOrder};

#[gpui_storybook::story("Table ordering")]
#[derive(gpui_storybook::StoryControls)]
pub struct SortEditorStory {
    focus: FocusHandle,
    ordering: SortOrder,
}
impl Focusable for SortEditorStory {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl gpui_storybook::Story for SortEditorStory {
    fn title(_: &App) -> String {
        "Ordered sort keys".into()
    }
    fn new_view(_: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self {
            focus: cx.focus_handle(),
            ordering: SortOrder::new(vec![
                SortClause::new("category", SortDirection::Ascending),
                SortClause::new("score", SortDirection::Descending),
            ])
            .unwrap(),
        })
    }
}
impl Render for SortEditorStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity();
        SortEditor::new(
            "sort-editor-story",
            vec![
                SortColumnOption::new("category", "Category"),
                SortColumnOption::new("score", "Score"),
                SortColumnOption::new("created", "Created"),
            ],
            self.ordering.clone(),
            move |ordering, _, cx| {
                owner.update(cx, |owner, cx| {
                    owner.ordering = ordering;
                    cx.notify();
                });
            },
        )
        .unwrap()
    }
}
