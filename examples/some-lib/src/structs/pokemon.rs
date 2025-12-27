use es_fluent::{EsFluentKv, EsFluentThis};
use gpui::{Context, Window};
use gpui_component::IconName;
use gpui_component::table::TableState;
use gpui_table::components::{FacetedFilter, NumberRangeFilter, TextFilter};
use gpui_table::{Filterable, GpuiTable, TableCell};
use serde::Deserialize;

/// Pokemon types from the PokeAPI
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    Hash,
    es_fluent::EsFluent,
    Filterable,
    TableCell,
    strum::EnumIter,
)]
#[filter(fluent)]
pub enum PokemonType {
    #[default]
    #[filter(icon = IconName::Minus)]
    Normal,
    #[filter(icon = IconName::TriangleAlert)]
    Fire,
    #[filter(icon = IconName::ArrowDown)]
    Water,
    #[filter(icon = IconName::Palette)]
    Grass,
    #[filter(icon = IconName::Star)]
    Electric,
    #[filter(icon = IconName::ArrowUp)]
    Ice,
    #[filter(icon = IconName::User)]
    Fighting,
    #[filter(icon = IconName::Settings)]
    Poison,
    #[filter(icon = IconName::ChartPie)]
    Ground,
    #[filter(icon = IconName::ArrowUp)]
    Flying,
    #[filter(icon = IconName::Moon)]
    Psychic,
    #[filter(icon = IconName::Search)]
    Bug,
    #[filter(icon = IconName::Settings)]
    Rock,
    #[filter(icon = IconName::Moon)]
    Ghost,
    #[filter(icon = IconName::Star)]
    Dragon,
    #[filter(icon = IconName::Moon)]
    Dark,
    #[filter(icon = IconName::Settings)]
    Steel,
    #[filter(icon = IconName::Star)]
    Fairy,
    /// Represents "None" for secondary type
    #[filter(icon = IconName::CircleX)]
    None,
}

impl PokemonType {
    pub fn from_api_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "normal" => Self::Normal,
            "fire" => Self::Fire,
            "water" => Self::Water,
            "grass" => Self::Grass,
            "electric" => Self::Electric,
            "ice" => Self::Ice,
            "fighting" => Self::Fighting,
            "poison" => Self::Poison,
            "ground" => Self::Ground,
            "flying" => Self::Flying,
            "psychic" => Self::Psychic,
            "bug" => Self::Bug,
            "rock" => Self::Rock,
            "ghost" => Self::Ghost,
            "dragon" => Self::Dragon,
            "dark" => Self::Dark,
            "steel" => Self::Steel,
            "fairy" => Self::Fairy,
            _ => Self::Normal,
        }
    }
}

/// A Pokemon entry for the table - combines data from PokeAPI
#[derive(Clone, Debug, EsFluentKv, EsFluentThis, GpuiTable)]
#[fluent_this(origin, members)]
#[fluent_kv(keys = ["description", "label"])]
#[gpui_table(fluent = "label")]
#[gpui_table(load_more = "Self::load_more_pokemon")]
#[gpui_table(load_more_threshold = 20)]
pub struct Pokemon {
    /// Pokemon ID from the API
    #[gpui_table(sortable, width = 60., resizable = false, movable = false)]
    pub id: u32,

    /// Pokemon name
    #[gpui_table(sortable, width = 150., filter = TextFilter)]
    pub name: String,

    /// Primary type
    #[gpui_table(width = 100., filter = FacetedFilter)]
    pub primary_type: PokemonType,

    /// Secondary type (None if single-type Pokemon)
    #[gpui_table(width = 100., filter = FacetedFilter)]
    pub secondary_type: PokemonType,

    /// Height in decimeters
    #[gpui_table(sortable, width = 80., filter = NumberRangeFilter)]
    pub height: u32,

    /// Weight in hectograms
    #[gpui_table(sortable, width = 80., filter = NumberRangeFilter)]
    pub weight: u32,

    /// Base experience
    #[gpui_table(sortable, width = 100., filter = NumberRangeFilter)]
    pub base_experience: u32,

    /// HP stat
    #[gpui_table(sortable, width = 60., filter = NumberRangeFilter)]
    pub hp: u32,

    /// Attack stat
    #[gpui_table(sortable, width = 70., filter = NumberRangeFilter)]
    pub attack: u32,

    /// Defense stat
    #[gpui_table(sortable, width = 70., filter = NumberRangeFilter)]
    pub defense: u32,

    /// Speed stat
    #[gpui_table(sortable, width = 70., filter = NumberRangeFilter)]
    pub speed: u32,
}

// ============================================================================
// PokeAPI Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct PokeApiListResponse {
    pub count: u32,
    pub next: Option<String>,
    #[allow(dead_code)]
    pub previous: Option<String>,
    pub results: Vec<PokeApiListItem>,
}

#[derive(Debug, Deserialize)]
pub struct PokeApiListItem {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct PokeApiPokemon {
    pub id: u32,
    pub name: String,
    pub height: u32,
    pub weight: u32,
    pub base_experience: Option<u32>,
    pub types: Vec<PokeApiTypeSlot>,
    pub stats: Vec<PokeApiStat>,
}

#[derive(Debug, Deserialize)]
pub struct PokeApiTypeSlot {
    pub slot: u32,
    #[serde(rename = "type")]
    pub type_info: PokeApiTypeInfo,
}

#[derive(Debug, Deserialize)]
pub struct PokeApiTypeInfo {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct PokeApiStat {
    pub base_stat: u32,
    pub stat: PokeApiStatInfo,
}

#[derive(Debug, Deserialize)]
pub struct PokeApiStatInfo {
    pub name: String,
}

impl Pokemon {
    /// Convert from API response to our Pokemon struct
    pub fn from_api(api: PokeApiPokemon) -> Self {
        let mut primary_type = PokemonType::Normal;
        let mut secondary_type = PokemonType::None;

        for type_slot in &api.types {
            let ptype = PokemonType::from_api_name(&type_slot.type_info.name);
            if type_slot.slot == 1 {
                primary_type = ptype;
            } else if type_slot.slot == 2 {
                secondary_type = ptype;
            }
        }

        let mut hp = 0;
        let mut attack = 0;
        let mut defense = 0;
        let mut speed = 0;

        for stat in &api.stats {
            match stat.stat.name.as_str() {
                "hp" => hp = stat.base_stat,
                "attack" => attack = stat.base_stat,
                "defense" => defense = stat.base_stat,
                "speed" => speed = stat.base_stat,
                _ => {},
            }
        }

        Self {
            id: api.id,
            name: titlecase(&api.name),
            primary_type,
            secondary_type,
            height: api.height,
            weight: api.weight,
            base_experience: api.base_experience.unwrap_or(0),
            hp,
            attack,
            defense,
            speed,
        }
    }
}

/// Convert a string to title case (first letter uppercase)
fn titlecase(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

// ============================================================================
// Table Delegate Implementation for API Loading
// ============================================================================

impl PokemonTableDelegate {
    /// Current offset for pagination
    pub fn current_offset(&self) -> u32 {
        self.rows.len() as u32
    }

    /// Load more Pokemon from the API
    pub fn load_more_pokemon(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        if self.loading || self.eof {
            return;
        }

        self.loading = true;
        cx.notify();

        let offset = self.current_offset();
        let batch_size = 20u32;

        // Clone filter values for use in async block
        let filters = self.filters.clone();

        cx.spawn(async move |view, cx| {
            // Fetch the list of Pokemon
            let list_url = format!(
                "https://pokeapi.co/api/v2/pokemon?offset={}&limit={}",
                offset, batch_size
            );

            let result: Result<Vec<Pokemon>, String> = async {
                // Fetch list
                let list_response = reqwest::get(&list_url)
                    .await
                    .map_err(|e| format!("Failed to fetch list: {}", e))?;

                let list: PokeApiListResponse = list_response
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse list: {}", e))?;

                // Fetch details for each Pokemon concurrently
                let client = reqwest::Client::new();
                let mut pokemon_list = Vec::new();

                for item in list.results {
                    let response = client
                        .get(&item.url)
                        .send()
                        .await
                        .map_err(|e| format!("Failed to fetch {}: {}", item.name, e))?;

                    let api_pokemon: PokeApiPokemon = response
                        .json()
                        .await
                        .map_err(|e| format!("Failed to parse {}: {}", item.name, e))?;

                    pokemon_list.push(Pokemon::from_api(api_pokemon));
                }

                Ok(pokemon_list)
            }
            .await;

            _ = cx.update(|cx| {
                view.update(cx, |table, cx| {
                    let delegate = table.delegate_mut();

                    match result {
                        Ok(new_pokemon) => {
                            // Apply client-side filtering
                            // In a real app with a searchable API, you'd pass filters to the API
                            let filtered: Vec<Pokemon> = new_pokemon
                                .into_iter()
                                .filter(|p| Self::matches_filters(p, &filters))
                                .collect();

                            delegate.rows.extend(filtered);

                            // PokeAPI has ~1300 Pokemon, stop at 500 for demo
                            if delegate.rows.len() >= 500 || offset + batch_size >= 1300 {
                                delegate.eof = true;
                            }
                        },
                        Err(e) => {
                            eprintln!("Error loading Pokemon: {}", e);
                            // On error, mark as EOF to prevent infinite retries
                            delegate.eof = true;
                        },
                    }

                    delegate.loading = false;
                    cx.notify();
                })
                .unwrap();
            });
        })
        .detach();
    }

    /// Check if a Pokemon matches the given filters
    fn matches_filters(pokemon: &Pokemon, filters: &PokemonFilters) -> bool {
        // Text filter on name
        if !filters.name.is_empty()
            && !pokemon
                .name
                .to_lowercase()
                .contains(&filters.name.to_lowercase())
        {
            return false;
        }

        // Primary type filter
        if !filters.primary_type.is_empty() {
            let type_str = format!("{:?}", pokemon.primary_type);
            if !filters.primary_type.contains(&type_str) {
                return false;
            }
        }

        // Secondary type filter
        if !filters.secondary_type.is_empty() {
            let type_str = format!("{:?}", pokemon.secondary_type);
            if !filters.secondary_type.contains(&type_str) {
                return false;
            }
        }

        // Number range filters
        Self::check_range(pokemon.height as f64, filters.height)
            && Self::check_range(pokemon.weight as f64, filters.weight)
            && Self::check_range(pokemon.base_experience as f64, filters.base_experience)
            && Self::check_range(pokemon.hp as f64, filters.hp)
            && Self::check_range(pokemon.attack as f64, filters.attack)
            && Self::check_range(pokemon.defense as f64, filters.defense)
            && Self::check_range(pokemon.speed as f64, filters.speed)
    }

    fn check_range(value: f64, range: (Option<f64>, Option<f64>)) -> bool {
        if let (Some(min), _) = range {
            if value < min {
                return false;
            }
        }
        if let (_, Some(max)) = range {
            if value > max {
                return false;
            }
        }
        true
    }

    /// Reset and reload data (call when filters change)
    pub fn reset_and_reload(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>) {
        self.rows.clear();
        self.eof = false;
        self.loading = false;
        self.load_more_pokemon(window, cx);
    }
}
