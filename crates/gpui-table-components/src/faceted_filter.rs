use gpui::{
    prelude::*, App, Context, Entity, FocusHandle, IntoElement, Render, VisualContext, Window,
};
use gpui_component::{
    badge::Badge, button::Button, checkbox::Checkbox, divider::Divider, label::Label,
    popover::Popover, scroll::ScrollableElement, v_flex, Icon, IconName,
};
use gpui_table_core::filter::FacetedFilterOption;
use std::collections::HashSet;
use std::rc::Rc;

pub struct FacetedFilter {
    title: String,
    options: Vec<FacetedFilterOption>,
    selected_values: HashSet<String>,
    on_change: Rc<dyn Fn(HashSet<String>, &mut Window, &mut App) + 'static>,
    focus_handle: FocusHandle,
}

impl FacetedFilter {
    pub fn build(
        title: impl Into<String>,
        options: Vec<FacetedFilterOption>,
        selected_values: HashSet<String>,
        on_change: impl Fn(HashSet<String>, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();

        cx.new(|cx| Self {
            title,
            options,
            selected_values,
            on_change: Rc::new(on_change),
            focus_handle: cx.focus_handle(),
        })
    }

    fn toggle_option(
        &mut self,
        value: String,
        checked: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if checked {
            self.selected_values.insert(value);
        } else {
            self.selected_values.remove(&value);
        }
        (self.on_change)(self.selected_values.clone(), window, cx);
        cx.notify();
    }

    fn clear_filters(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_values.clear();
        (self.on_change)(self.selected_values.clone(), window, cx);
        cx.notify();
    }
}

impl Render for FacetedFilter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = self.title.clone();
        let selected_count = self.selected_values.len();
        let has_selection = selected_count > 0;

        let view = cx.entity().clone();
        let options = self.options.clone();
        let selected_values = self.selected_values.clone();

        let trigger = Button::new("trigger")
            .label(title)
            .icon(IconName::Plus)
            .when(has_selection, |b| {
                b.child(Badge::new().child(selected_count.to_string()))
            });

        Popover::new("filter-popover")
            .trigger(trigger)
            .content(move |_, _window, _cx| {
                let options_view = v_flex().gap_1().children(options.iter().map(|opt| {
                    let is_selected = selected_values.contains(&opt.value);
                    let val = opt.value.clone();
                    let view = view.clone();

                    let mut checkbox = Checkbox::new(format!("opt-{}", val))
                        .label(opt.label.clone())
                        .checked(is_selected)
                        .on_click(move |checked, window, cx| {
                            view.update(cx, |this, cx| {
                                this.toggle_option(val.clone(), *checked, window, cx);
                            });
                        });

                    checkbox
                }));

                v_flex()
                    .w_64()
                    .p_2()
                    .gap_2()
                    .child(Label::new("Search..."))
                    .child(
                        v_flex()
                            .id("options-list")
                            .h_64()
                            .overflow_y_scrollbar()
                            .child(options_view),
                    )
                    .when(has_selection, |this| {
                        let view = view.clone();
                        this.child(Divider::horizontal()).child(
                            Button::new("clear").label("Clear filters").on_click(
                                move |_, window, cx| {
                                    view.update(cx, |this, cx| {
                                        this.clear_filters(window, cx);
                                    });
                                },
                            ),
                        )
                    })
            })
    }
}
