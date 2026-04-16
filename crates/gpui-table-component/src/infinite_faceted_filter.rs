use crate::TableFilterComponent;
use gpui::{
    App, Context, Entity, IntoElement, Render, StyleRefinement, Subscription, Window, div,
    prelude::*, px,
};
use gpui_component::{
    Icon, IconName, IndexPath, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    divider::Divider,
    popover::Popover,
    select::{Select, SelectEvent, SelectState},
    v_flex,
};
use gpui_form_component::infinite_select::{
    InfiniteSelect, InfiniteSelectItem, InfiniteSelectPath, build_from_path, to_select_items,
};
use std::rc::Rc;

type InfiniteSelectState<T> = SelectState<Vec<InfiniteSelectItem<T>>>;

pub struct InfiniteFacetedFilter<T: InfiniteSelect + PartialEq + Send> {
    title: Rc<dyn Fn() -> String>,
    selected_value: Option<T>,
    selection_path: InfiniteSelectPath,
    trigger_style: StyleRefinement,
    popover_style: StyleRefinement,
    master_select: Option<Entity<InfiniteSelectState<T>>>,
    child_selects: Vec<Entity<InfiniteSelectState<T>>>,
    master_subscription: Option<Subscription>,
    child_subscriptions: Vec<Subscription>,
    on_change: Rc<dyn Fn(Option<T>, &mut Window, &mut App) + 'static>,
}

impl<T> TableFilterComponent for InfiniteFacetedFilter<T>
where
    T: InfiniteSelect + PartialEq + Send,
{
    type Value = Option<T>;

    const FILTER_TYPE: gpui_table_schema::registry::RegistryFilterType =
        gpui_table_schema::registry::RegistryFilterType::InfiniteFaceted;

    fn new(
        title: impl Into<String>,
        value: Self::Value,
        on_change: impl Fn(Self::Value, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();
        Self::new_with_title(Rc::new(move || title.clone()), value, on_change, cx)
    }
}

impl<T> InfiniteFacetedFilter<T>
where
    T: InfiniteSelect + PartialEq + Send,
{
    /// Create an infinite faceted filter with a fixed title.
    pub fn new(
        title: impl Into<String>,
        value: Option<T>,
        on_change: impl Fn(Option<T>, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let title = title.into();
        Self::new_with_title(Rc::new(move || title.clone()), value, on_change, cx)
    }

    fn new_with_title(
        title: Rc<dyn Fn() -> String>,
        value: Option<T>,
        on_change: impl Fn(Option<T>, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        let selection_path = value
            .as_ref()
            .and_then(build_path_to_value::<T>)
            .unwrap_or_default();
        let selected_value = value.and_then(|selected| {
            let path = build_path_to_value(&selected)?;
            is_complete_selection(&selected, path.len()).then_some(selected)
        });

        cx.new(|_cx| Self {
            title,
            selected_value,
            selection_path,
            trigger_style: StyleRefinement::default(),
            popover_style: StyleRefinement::default(),
            master_select: None,
            child_selects: Vec::new(),
            master_subscription: None,
            child_subscriptions: Vec::new(),
            on_change: Rc::new(on_change),
        })
    }

    /// Create an infinite faceted filter with a reactive title provider.
    pub fn new_for(
        title: impl Fn() -> String + 'static,
        value: Option<T>,
        on_change: impl Fn(Option<T>, &mut Window, &mut App) + 'static,
        cx: &mut App,
    ) -> Entity<Self> {
        Self::new_with_title(Rc::new(title), value, on_change, cx)
    }

    fn ensure_selects(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.master_select.is_none() {
            let selected_index = self.selection_path.get(0).map(IndexPath::new);
            let master_select =
                cx.new(|cx| SelectState::new(to_select_items::<T>(), selected_index, window, cx));
            let subscription =
                cx.subscribe_in(&master_select, window, Self::on_master_select_event);
            self.master_subscription = Some(subscription);
            self.master_select = Some(master_select);
        }

        self.rebuild_child_selects(window, cx);
    }

    fn on_master_select_event(
        &mut self,
        this: &Entity<InfiniteSelectState<T>>,
        event: &SelectEvent<Vec<InfiniteSelectItem<T>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            SelectEvent::Confirm(Some(_)) => {
                if let Some(index) = this.read(cx).selected_index(cx) {
                    self.selection_path.set(0, index.row);
                    self.apply_selection_path(window, cx);
                }
            },
            SelectEvent::Confirm(None) => self.reset_inner(true, window, cx),
        }
    }

    fn on_child_select_event(
        &mut self,
        this: &Entity<InfiniteSelectState<T>>,
        event: &SelectEvent<Vec<InfiniteSelectItem<T>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(level) = self
            .child_selects
            .iter()
            .position(|select| select == this)
            .map(|index| index + 1)
        else {
            return;
        };

        match event {
            SelectEvent::Confirm(Some(_)) => {
                if let Some(index) = this.read(cx).selected_index(cx) {
                    self.selection_path.set(level, index.row);
                    self.apply_selection_path(window, cx);
                }
            },
            SelectEvent::Confirm(None) => {
                self.selection_path.truncate(level);
                self.apply_selection_path(window, cx);
            },
        }
    }

    fn apply_selection_path(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_value = self.active_value_from_path();
        self.rebuild_child_selects(window, cx);
        (self.on_change)(self.selected_value.clone(), window, cx);
        cx.notify();
    }

    fn active_value_from_path(&self) -> Option<T> {
        let candidate = build_from_path::<T>(&self.selection_path)?;
        is_complete_selection(&candidate, self.selection_path.len()).then_some(candidate)
    }

    fn rebuild_child_selects(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.child_selects.clear();
        self.child_subscriptions.clear();

        let Some(mut current_value) = build_from_path::<T>(&self.selection_path) else {
            return;
        };

        let max_depth = required_selection_len(&current_value).saturating_sub(1);

        for level in 0..max_depth {
            let (child_names, has_more) = if level == 0 {
                (
                    current_value.child_variant_names(),
                    current_value.has_inner(),
                )
            } else {
                (
                    current_value.inner_child_variant_names(),
                    current_value.inner_has_inner(),
                )
            };

            if !has_more || child_names.is_empty() {
                break;
            }

            let items: Vec<_> = child_names
                .iter()
                .enumerate()
                .filter_map(|(index, label)| {
                    let next_value = if level == 0 {
                        current_value.set_child_by_index(index)
                    } else {
                        current_value.inner_set_child_by_index(index)
                    }?;

                    Some(InfiniteSelectItem::new(next_value, (*label).to_string()))
                })
                .collect();

            let selected_index = self.selection_path.get(level + 1).map(IndexPath::new);
            let child_select =
                cx.new(|cx| SelectState::new(items.clone(), selected_index, window, cx));
            let subscription = cx.subscribe_in(&child_select, window, Self::on_child_select_event);

            self.child_subscriptions.push(subscription);
            self.child_selects.push(child_select);

            let Some(selected_row) = self.selection_path.get(level + 1) else {
                break;
            };
            let Some(selected_item) = items.get(selected_row) else {
                break;
            };

            current_value = selected_item.get_value().clone();
        }
    }

    fn reset_inner(&mut self, notify_change: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.selection_path = InfiniteSelectPath::new();
        self.selected_value = None;
        self.child_selects.clear();
        self.child_subscriptions.clear();

        if let Some(master_select) = &self.master_select {
            master_select.update(cx, |select, cx| {
                select.set_selected_index(None, window, cx);
            });
        }

        if notify_change {
            (self.on_change)(None, window, cx);
        }

        cx.notify();
    }

    /// Reset the selection and notify via callback.
    pub fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_inner(true, window, cx);
    }

    /// Reset the selection without invoking callback.
    pub fn reset_silent(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_inner(false, window, cx);
    }

    /// Get the current selected value, if the selection is complete.
    pub fn value(&self) -> Option<T> {
        self.selected_value.clone()
    }
}

impl<T> Render for InfiniteFacetedFilter<T>
where
    T: InfiniteSelect + PartialEq + Send,
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_selects(window, cx);

        let title = (self.title)();
        let has_selection = !self.selection_path.is_empty();
        let selected_display = path_display::<T>(&self.selection_path);
        let trigger_icon = if has_selection {
            IconName::CircleX
        } else {
            IconName::Plus
        };
        let trigger_style = self.trigger_style.clone();
        let popover_style = self.popover_style.clone();
        let view = cx.entity().clone();
        let clear_view = view.clone();
        let master_select = self
            .master_select
            .clone()
            .expect("master select must be initialized before render");
        let child_selects = self.child_selects.clone();
        let child_placeholders = build_child_placeholders::<T>(&self.selection_path);
        let trigger_title = title.clone();

        let trigger = Button::new("infinite-faceted-filter-trigger")
            .outline()
            .refine_style(&trigger_style)
            .child(
                div()
                    .id("clear-icon")
                    .when(has_selection, |this| {
                        this.cursor_pointer()
                            .rounded_sm()
                            .hover(|style| style.opacity(1.0))
                            .opacity(0.7)
                            .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                                clear_view.update(cx, |this, cx| {
                                    this.reset_inner(true, window, cx);
                                });
                            })
                    })
                    .child(Icon::new(trigger_icon).xsmall()),
            )
            .child(trigger_title.clone())
            .when_some(selected_display.clone(), |button, display| {
                button
                    .child(Divider::vertical().h(px(16.)).mx_1())
                    .child(div().max_w_56().truncate().child(display))
            });

        Popover::new("infinite-faceted-filter-popover")
            .trigger(trigger)
            .content(move |_, _window, _cx| {
                let clear_view = view.clone();
                let title = trigger_title.clone();
                let child_placeholders = child_placeholders.clone();

                v_flex()
                    .w_72()
                    .gap_2()
                    .p_3()
                    .refine_style(&popover_style)
                    .child(
                        Select::new(&master_select)
                            .cleanable(true)
                            .small()
                            .placeholder(title),
                    )
                    .children(child_selects.iter().enumerate().map(|(depth, select)| {
                        let placeholder = child_placeholders
                            .get(depth)
                            .cloned()
                            .unwrap_or_else(|| format!("Level {}", depth + 2));

                        Select::new(select)
                            .cleanable(true)
                            .small()
                            .placeholder(placeholder)
                            .into_any_element()
                    }))
                    .when(has_selection, |this| {
                        this.child(Divider::horizontal()).child(
                            Button::new("clear-infinite-faceted-filter")
                                .ghost()
                                .w_full()
                                .justify_center()
                                .label("Clear filter")
                                .on_click(move |_, window, cx| {
                                    clear_view.update(cx, |this, cx| {
                                        this.reset_inner(true, window, cx);
                                    });
                                }),
                        )
                    })
                    .into_any_element()
            })
    }
}

fn required_selection_len<T>(value: &T) -> usize
where
    T: InfiniteSelect,
{
    if value.has_inner() {
        value.child_depth() + 2
    } else {
        1
    }
}

fn is_complete_selection<T>(value: &T, path_len: usize) -> bool
where
    T: InfiniteSelect,
{
    path_len == required_selection_len(value)
}

fn build_path_to_value<T>(value: &T) -> Option<InfiniteSelectPath>
where
    T: InfiniteSelect + PartialEq,
{
    T::variants()
        .into_iter()
        .enumerate()
        .find_map(|(index, root)| {
            let mut path = InfiniteSelectPath::new();
            path.set(0, index);
            find_value_path(root, value, path, 0)
        })
}

fn find_value_path<T>(
    current: T,
    target: &T,
    path: InfiniteSelectPath,
    level: usize,
) -> Option<InfiniteSelectPath>
where
    T: InfiniteSelect + PartialEq,
{
    if &current == target && is_complete_selection(&current, path.len()) {
        return Some(path);
    }

    let (child_names, has_more) = if level == 0 {
        (current.child_variant_names(), current.has_inner())
    } else {
        (
            current.inner_child_variant_names(),
            current.inner_has_inner(),
        )
    };

    if !has_more || child_names.is_empty() {
        return None;
    }

    (0..child_names.len()).find_map(|index| {
        let next = if level == 0 {
            current.set_child_by_index(index)
        } else {
            current.inner_set_child_by_index(index)
        }?;

        let mut next_path = path.clone();
        next_path.set(level + 1, index);
        find_value_path(next, target, next_path, level + 1)
    })
}

fn build_child_placeholders<T>(path: &InfiniteSelectPath) -> Vec<String>
where
    T: InfiniteSelect,
{
    let Some(current_value) = build_from_path::<T>(path) else {
        return Vec::new();
    };

    let child_levels = required_selection_len(&current_value).saturating_sub(1);
    (0..child_levels)
        .map(|depth| {
            current_value
                .child_label_at_depth(depth)
                .map(|label| label.to_string())
                .filter(|label| !label.is_empty())
                .unwrap_or_else(|| format!("Level {}", depth + 2))
        })
        .collect()
}

fn path_display<T>(path: &InfiniteSelectPath) -> Option<String>
where
    T: InfiniteSelect,
{
    let root_index = path.get(0)?;
    let mut current = T::variants().get(root_index)?.clone();
    let mut parts = vec![current.variant_name().to_string()];

    for level in 1..path.len() {
        let row_index = path.get(level)?;
        let label = if level == 1 {
            current.child_variant_names().get(row_index).copied()
        } else {
            current.inner_child_variant_names().get(row_index).copied()
        }?;
        parts.push(label.to_string());

        current = if level == 1 {
            current.set_child_by_index(row_index)?
        } else {
            current.inner_set_child_by_index(row_index)?
        };
    }

    Some(parts.join(" / "))
}
