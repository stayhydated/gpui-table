use super::*;

/// Extension trait for chainable configuration on `Entity<NumberRangeFilter>`.
pub trait NumberRangeFilterExt: Sized {
    /// Set the range bounds for the slider (chainable).
    ///
    /// # Example
    /// ```ignore
    /// NumberRangeFilter::build("Price", (None, None), on_change, cx)
    ///     .range(Decimal::ZERO, Decimal::new(1000, 0), cx)
    ///     .step(Decimal::TEN, cx)
    /// ```
    fn range(self, min: Decimal, max: Decimal, cx: &mut App) -> Self;

    /// Set the step size for increment/decrement (chainable).
    /// Default is 1% of the range.
    fn step(self, step: Decimal, cx: &mut App) -> Self;
    /// Set style refinement for the trigger button.
    fn trigger_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
    /// Set style refinement for the popover root content.
    fn popover_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
    /// Set style refinement for the min/max input row.
    fn inputs_row_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
    /// Set style refinement for the minimum input.
    fn min_input_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
    /// Set style refinement for the maximum input.
    fn max_input_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
    /// Set style refinement for the localized "between" label.
    fn between_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
    /// Set style refinement for the slider.
    fn slider_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
    /// Set style refinement for the clear button in the popover.
    fn clear_button_style(self, _style: StyleRefinement, _cx: &mut App) -> Self {
        self
    }
}

impl NumberRangeFilterExt for Entity<NumberRangeFilter> {
    fn range(self, min: Decimal, max: Decimal, cx: &mut App) -> Self {
        self.update(cx, |this, cx| {
            this.range_min = min;
            this.range_max = max;
            this.range_is_explicit = true;
            if let Some(current_min) = this.min {
                this.min = Some(current_min.clamp(min, max));
            }
            if let Some(current_max) = this.max {
                this.max = Some(current_max.clamp(min, max));
            }
            if let Some(slider) = &this.slider_state {
                this.sync_slider_state(slider, cx);
            }
        });
        self
    }

    fn step(self, step: Decimal, cx: &mut App) -> Self {
        self.update(cx, |this, _cx| {
            this.step_size = Some(step);
        });
        self
    }

    fn trigger_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _cx| {
            this.trigger_style = style;
        });
        self
    }

    fn popover_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _cx| {
            this.popover_style = style;
        });
        self
    }

    fn inputs_row_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _cx| {
            this.inputs_row_style = style;
        });
        self
    }

    fn min_input_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _cx| {
            this.min_input_style = style;
        });
        self
    }

    fn max_input_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _cx| {
            this.max_input_style = style;
        });
        self
    }

    fn between_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _cx| {
            this.between_style = style;
        });
        self
    }

    fn slider_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _cx| {
            this.slider_style = style;
        });
        self
    }

    fn clear_button_style(self, style: StyleRefinement, cx: &mut App) -> Self {
        self.update(cx, |this, _cx| {
            this.clear_button_style = style;
        });
        self
    }
}
