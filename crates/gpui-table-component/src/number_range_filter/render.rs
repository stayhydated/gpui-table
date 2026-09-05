use super::*;

fn app<'a, T>(cx: &'a Context<'_, T>) -> &'a App {
    cx.borrow()
}

impl Render for NumberRangeFilter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Ensure input states exist
        self.ensure_inputs(window, cx);

        let min_placeholder = Self::min_placeholder_text(cx);
        let max_placeholder = Self::max_placeholder_text(cx);

        // Keep placeholders reactive to locale changes.
        if let Some(min_input) = &self.min_input
            && self.last_min_placeholder.as_deref() != Some(min_placeholder.as_str())
        {
            self.last_min_placeholder = Some(min_placeholder.clone());
            let min_placeholder_for_input = min_placeholder.clone();
            min_input.update(cx, |input, cx| {
                input.set_placeholder(min_placeholder_for_input, window, cx);
            });
        }
        if let Some(max_input) = &self.max_input
            && self.last_max_placeholder.as_deref() != Some(max_placeholder.as_str())
        {
            self.last_max_placeholder = Some(max_placeholder.clone());
            let max_placeholder_for_input = max_placeholder.clone();
            max_input.update(cx, |input, cx| {
                input.set_placeholder(max_placeholder_for_input, window, cx);
            });
        }

        // Sync components based on what changed last
        self.sync_components(window, cx);

        // Apply pending changes now that we have window access
        if self.pending_apply {
            self.pending_apply = false;
            (self.on_change)((self.min, self.max), window, cx);
        }

        let title = (self.title)(app(cx));
        let has_value = self.has_value();
        let range_display = self.format_range();
        let view = cx.entity();
        let (Some(min_input), Some(max_input), Some(slider_state)) = (
            self.min_input.clone(),
            self.max_input.clone(),
            self.slider_state.clone(),
        ) else {
            return gpui_kit::div().into_any_element();
        };
        let between = Self::between_text(cx);
        let trigger_style = self.trigger_style.clone();
        let popover_style = self.popover_style.clone();
        let inputs_row_style = self.inputs_row_style.clone();
        let min_input_style = self.min_input_style.clone();
        let max_input_style = self.max_input_style.clone();
        let between_style = self.between_style.clone();
        let slider_style = self.slider_style.clone();
        let clear_button_style = self.clear_button_style.clone();
        let between_width_px = Self::between_width_px(&between);
        let input_width_px =
            Self::input_width_px(between_width_px, &min_placeholder, &max_placeholder);
        let row_width_px = Self::row_width_px(input_width_px, between_width_px);
        let popover_width_px = Self::popover_width_px(row_width_px);

        // Icon: CircleX when has value (to clear), Plus otherwise
        let trigger_icon = if has_value {
            IconName::CircleX
        } else {
            IconName::Plus
        };

        let clear_view = view.clone();
        let trigger = Button::new("number-range-trigger")
            .outline()
            .refine_style(&trigger_style)
            .child(
                gpui_kit::div()
                    .id("clear-icon")
                    .when(has_value, |this| {
                        this.cursor_pointer()
                            .rounded_sm()
                            .hover(|s| s.opacity(1.0))
                            .opacity(0.7)
                            .on_mouse_down(gpui_kit::MouseButton::Left, move |_, window, cx| {
                                clear_view.update(cx, |this, cx| {
                                    this.clear(window, cx);
                                });
                            })
                    })
                    .child(Icon::new(trigger_icon).xsmall()),
            )
            .child(title)
            .when(has_value, |b| {
                b.child(Separator::vertical().h(px(16.)).mx_1())
                    .child(range_display)
            });

        Popover::new("number-range-popover")
            .trigger(trigger)
            .content(move |_, _window, cx| {
                let clear_view_inner = view.clone();
                let between_style = between_style.clone();
                let clear_button_style = clear_button_style.clone();
                v_flex()
                    .p_3()
                    .gap_3()
                    .w(px(popover_width_px))
                    .refine_style(&popover_style)
                    .child(
                        h_flex()
                            .w(px(row_width_px))
                            .gap_2()
                            .items_center()
                            .refine_style(&inputs_row_style)
                            .child(
                                gpui_kit::div().w(px(input_width_px)).child(
                                    NumberInput::new(&min_input)
                                        .small()
                                        .w_full()
                                        .refine_style(&min_input_style),
                                ),
                            )
                            .child(
                                gpui_kit::div()
                                    .w(px(between_width_px))
                                    .flex()
                                    .justify_center()
                                    .child(
                                        gpui_kit::div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .refine_style(&between_style)
                                            .child(between.clone()),
                                    ),
                            )
                            .child(
                                gpui_kit::div().w(px(input_width_px)).child(
                                    NumberInput::new(&max_input)
                                        .small()
                                        .w_full()
                                        .refine_style(&max_input_style),
                                ),
                            ),
                    )
                    .child(Slider::new(&slider_state).refine_style(&slider_style))
                    .when(has_value, |this| {
                        this.child(
                            Button::new("clear-btn")
                                .outline()
                                .small()
                                .w_full()
                                .label("Clear")
                                .refine_style(&clear_button_style)
                                .on_click(move |_, window, cx| {
                                    clear_view_inner.update(cx, |this, cx| {
                                        this.clear(window, cx);
                                    });
                                }),
                        )
                    })
            })
            .into_any_element()
    }
}
