use std::collections::HashSet;

use es_fluent::{ThisFtl as _, ToFluentString};
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, ParentElement, Render, Styled, Subscription,
    Window,
};
use gpui_component::{
    ActiveTheme, h_flex,
    table::{Table, TableState},
    v_flex,
};
use gpui_table::components::{FacetedFilter, NumberRangeFilter, TextFilter};
use gpui_table_components::TableFilterComponent;
use some_lib::structs::pokemon::{Pokemon, PokemonLabelKvFtl, PokemonTableDelegate, PokemonType};

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story]
pub struct PokemonStory {
    table: Entity<TableState<PokemonTableDelegate>>,

    // Text filter
    filter_name: Entity<TextFilter>,

    // Faceted filters for types
    filter_primary_type: Entity<FacetedFilter>,
    filter_secondary_type: Entity<FacetedFilter>,

    // Number range filters for stats
    filter_height: Entity<NumberRangeFilter>,
    filter_weight: Entity<NumberRangeFilter>,
    filter_base_experience: Entity<NumberRangeFilter>,
    filter_hp: Entity<NumberRangeFilter>,
    filter_attack: Entity<NumberRangeFilter>,
    filter_defense: Entity<NumberRangeFilter>,
    filter_speed: Entity<NumberRangeFilter>,

    _subscription: Subscription,
}

impl gpui_storybook::Story for PokemonStory {
    fn title() -> String {
        Pokemon::this_ftl()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl Focusable for PokemonStory {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}

impl PokemonStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = PokemonTableDelegate::new(vec![]);
        let table = cx.new(|cx| TableState::new(delegate, window, cx));

        // Trigger initial load
        table.update(cx, |table, cx| {
            table.delegate_mut().load_more_pokemon(window, cx);
        });

        // TextFilter: name
        let table_entity = table.clone();
        let filter_name = TextFilter::build(
            "Name",
            String::new(),
            move |value, window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.name = value;
                    table.delegate_mut().reset_and_reload(window, cx);
                });
            },
            cx,
        );

        // FacetedFilter: primary_type
        let table_entity = table.clone();
        let filter_primary_type = FacetedFilter::build_for::<PokemonType>(
            || PokemonLabelKvFtl::PrimaryType.to_fluent_string(),
            HashSet::new(),
            move |value, window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.primary_type = value;
                    table.delegate_mut().reset_and_reload(window, cx);
                });
            },
            cx,
        );

        // FacetedFilter: secondary_type
        let table_entity = table.clone();
        let filter_secondary_type = FacetedFilter::build_for::<PokemonType>(
            || PokemonLabelKvFtl::SecondaryType.to_fluent_string(),
            HashSet::new(),
            move |value, window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.secondary_type = value;
                    table.delegate_mut().reset_and_reload(window, cx);
                });
            },
            cx,
        );

        // NumberRangeFilter: height
        let table_entity = table.clone();
        let filter_height = NumberRangeFilter::build(
            "Height",
            (None, None),
            move |value, window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.height = value;
                    table.delegate_mut().reset_and_reload(window, cx);
                });
            },
            cx,
        );

        // NumberRangeFilter: weight
        let table_entity = table.clone();
        let filter_weight = NumberRangeFilter::build(
            "Weight",
            (None, None),
            move |value, window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.weight = value;
                    table.delegate_mut().reset_and_reload(window, cx);
                });
            },
            cx,
        );

        // NumberRangeFilter: base_experience
        let table_entity = table.clone();
        let filter_base_experience = NumberRangeFilter::build(
            "Base XP",
            (None, None),
            move |value, window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.base_experience = value;
                    table.delegate_mut().reset_and_reload(window, cx);
                });
            },
            cx,
        );

        // NumberRangeFilter: hp
        let table_entity = table.clone();
        let filter_hp = NumberRangeFilter::build(
            "HP",
            (None, None),
            move |value, window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.hp = value;
                    table.delegate_mut().reset_and_reload(window, cx);
                });
            },
            cx,
        );

        // NumberRangeFilter: attack
        let table_entity = table.clone();
        let filter_attack = NumberRangeFilter::build(
            "Attack",
            (None, None),
            move |value, window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.attack = value;
                    table.delegate_mut().reset_and_reload(window, cx);
                });
            },
            cx,
        );

        // NumberRangeFilter: defense
        let table_entity = table.clone();
        let filter_defense = NumberRangeFilter::build(
            "Defense",
            (None, None),
            move |value, window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.defense = value;
                    table.delegate_mut().reset_and_reload(window, cx);
                });
            },
            cx,
        );

        // NumberRangeFilter: speed
        let table_entity = table.clone();
        let filter_speed = NumberRangeFilter::build(
            "Speed",
            (None, None),
            move |value, window, cx| {
                table_entity.update(cx, |table, cx| {
                    table.delegate_mut().filters.speed = value;
                    table.delegate_mut().reset_and_reload(window, cx);
                });
            },
            cx,
        );

        let _subscription = cx.observe(&table, |_, _, cx| cx.notify());

        Self {
            table,
            filter_name,
            filter_primary_type,
            filter_secondary_type,
            filter_height,
            filter_weight,
            filter_base_experience,
            filter_hp,
            filter_attack,
            filter_defense,
            filter_speed,
            _subscription,
        }
    }
}

impl Render for PokemonStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let table = self.table.read(cx);
        let delegate = table.delegate();

        v_flex()
            .size_full()
            .gap_4()
            .p_4()
            // Name and Type Filters row
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        gpui::div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Name & Type Filters"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(self.filter_name.clone())
                            .child(self.filter_primary_type.clone())
                            .child(self.filter_secondary_type.clone()),
                    ),
            )
            // Physical Attributes row
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        gpui::div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Physical Attributes"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(self.filter_height.clone())
                            .child(self.filter_weight.clone())
                            .child(self.filter_base_experience.clone()),
                    ),
            )
            // Stats Filters row
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        gpui::div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Stats Filters"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(self.filter_hp.clone())
                            .child(self.filter_attack.clone())
                            .child(self.filter_defense.clone())
                            .child(self.filter_speed.clone()),
                    ),
            )
            // Status bar
            .child(
                h_flex()
                    .gap_4()
                    .child(format!("Pokemon Loaded: {}", delegate.rows.len()))
                    .child(if delegate.loading {
                        "Fetching from PokeAPI..."
                    } else {
                        "Idle"
                    })
                    .child(if delegate.eof {
                        "All data loaded"
                    } else {
                        "Scroll for more"
                    }),
            )
            // Table
            .child(
                Table::new(&self.table)
                    .stripe(true)
                    .scrollbar_visible(true, true),
            )
    }
}
