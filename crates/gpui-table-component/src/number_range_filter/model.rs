use rust_decimal::{Decimal, prelude::ToPrimitive as _};

const DEFAULT_RANGE_MIN_F32: f32 = 0.0;
const DEFAULT_RANGE_MAX_F32: f32 = 100.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BoundTextChange {
    Set(Decimal),
    Clear,
    Unchanged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StepDirection {
    Increment,
    Decrement,
}

pub(super) fn dynamic_range(min: Option<Decimal>, max: Option<Decimal>) -> (Decimal, Decimal) {
    let mut range_min = Decimal::ZERO;
    let mut range_max = Decimal::ONE_HUNDRED;
    for value in [min, max].into_iter().flatten() {
        range_min = range_min.min(value);
        range_max = range_max.max(value);
    }
    (range_min, range_max)
}

pub(super) fn bound_text_change(
    text: &str,
    range_is_explicit: bool,
    range_min: Decimal,
    range_max: Decimal,
) -> BoundTextChange {
    if text.is_empty() {
        return BoundTextChange::Clear;
    }

    let Ok(value) = text.parse::<Decimal>() else {
        return BoundTextChange::Unchanged;
    };
    BoundTextChange::Set(if range_is_explicit {
        value.clamp(range_min, range_max)
    } else {
        value
    })
}

pub(super) fn slider_values(
    min: Option<Decimal>,
    max: Option<Decimal>,
    range_min: Decimal,
    range_max: Decimal,
) -> (f32, f32, f32, f32) {
    let range_min = range_min.to_f32().unwrap_or(DEFAULT_RANGE_MIN_F32);
    let range_max = range_max.to_f32().unwrap_or(DEFAULT_RANGE_MAX_F32);
    let current_min = min
        .and_then(|value| value.to_f32())
        .unwrap_or(range_min)
        .clamp(range_min, range_max);
    let current_max = max
        .and_then(|value| value.to_f32())
        .unwrap_or(range_max)
        .clamp(range_min, range_max);

    (range_min, range_max, current_min, current_max)
}

pub(super) fn step_amount(
    configured: Option<Decimal>,
    range_min: Decimal,
    range_max: Decimal,
) -> Decimal {
    configured.unwrap_or((range_max - range_min) / Decimal::ONE_HUNDRED)
}

pub(super) fn stepped_value(
    current: Option<Decimal>,
    fallback: Decimal,
    step: Decimal,
    direction: StepDirection,
    explicit_range: Option<(Decimal, Decimal)>,
) -> Decimal {
    let current = current.unwrap_or(fallback);
    let value = match direction {
        StepDirection::Increment => current + step,
        StepDirection::Decrement => current - step,
    };
    explicit_range.map_or(value, |(min, max)| value.clamp(min, max))
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::{
        BoundTextChange, StepDirection, bound_text_change, dynamic_range, slider_values,
        step_amount, stepped_value,
    };

    #[test]
    fn dynamic_ranges_expand_around_values_and_keep_defaults() {
        assert_eq!(
            dynamic_range(None, None),
            (Decimal::ZERO, Decimal::ONE_HUNDRED)
        );
        assert_eq!(
            dynamic_range(Some(Decimal::from(-20)), Some(Decimal::from(250))),
            (Decimal::from(-20), Decimal::from(250))
        );
        assert_eq!(
            dynamic_range(Some(Decimal::from(20)), Some(Decimal::from(80))),
            (Decimal::ZERO, Decimal::ONE_HUNDRED)
        );
    }

    #[test]
    fn text_changes_distinguish_values_clears_and_invalid_input() {
        assert_eq!(
            bound_text_change("12.5", false, Decimal::ZERO, Decimal::TEN),
            BoundTextChange::Set(Decimal::new(125, 1))
        );
        assert_eq!(
            bound_text_change("12.5", true, Decimal::ZERO, Decimal::TEN),
            BoundTextChange::Set(Decimal::TEN)
        );
        assert_eq!(
            bound_text_change("", false, Decimal::ZERO, Decimal::TEN),
            BoundTextChange::Clear
        );
        assert_eq!(
            bound_text_change("twelve", false, Decimal::ZERO, Decimal::TEN),
            BoundTextChange::Unchanged
        );
    }

    #[test]
    fn slider_projection_supplies_defaults_and_clamps_values() {
        assert_eq!(
            slider_values(None, None, Decimal::ZERO, Decimal::ONE_HUNDRED),
            (0.0, 100.0, 0.0, 100.0)
        );
        assert_eq!(
            slider_values(
                Some(Decimal::from(-20)),
                Some(Decimal::from(120)),
                Decimal::ZERO,
                Decimal::ONE_HUNDRED,
            ),
            (0.0, 100.0, 0.0, 100.0)
        );
    }

    #[test]
    fn stepping_uses_configured_or_range_relative_amounts() {
        assert_eq!(
            step_amount(None, Decimal::ZERO, Decimal::from(200)),
            Decimal::from(2)
        );
        assert_eq!(
            step_amount(Some(Decimal::new(5, 1)), Decimal::ZERO, Decimal::TEN),
            Decimal::new(5, 1)
        );
        assert_eq!(
            stepped_value(
                Some(Decimal::from(9)),
                Decimal::ZERO,
                Decimal::from(2),
                StepDirection::Increment,
                Some((Decimal::ZERO, Decimal::TEN)),
            ),
            Decimal::TEN
        );
        assert_eq!(
            stepped_value(
                None,
                Decimal::TEN,
                Decimal::from(2),
                StepDirection::Decrement,
                None,
            ),
            Decimal::from(8)
        );
    }
}
