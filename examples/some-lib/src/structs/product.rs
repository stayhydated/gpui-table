use es_fluent::{EsFluentKv, EsFluentThis};
use gpui::{Context, Window};
use gpui_component::IconName;
use gpui_component::table::TableState;
use gpui_table::{Filterable, GpuiTable, TableCell};
use gpui_tokio::Tokio;
use heck::ToKebabCase;
use log::{debug, info, warn};
use serde::Deserialize;

/// Product categories from DummyJSON
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    Hash,
    Deserialize,
    es_fluent::EsFluent,
    Filterable,
    TableCell,
    strum::EnumIter,
)]
#[serde(rename_all = "kebab-case")]
#[filter(fluent)]
pub enum ProductCategory {
    #[default]
    #[filter(icon = IconName::Star)]
    Beauty,
    #[filter(icon = IconName::Star)]
    Fragrances,
    #[filter(icon = IconName::Settings)]
    Furniture,
    #[filter(icon = IconName::ChartPie)]
    Groceries,
    #[filter(icon = IconName::Palette)]
    HomeDecoration,
    #[filter(icon = IconName::Settings)]
    KitchenAccessories,
    #[filter(icon = IconName::Settings)]
    Laptops,
    #[filter(icon = IconName::User)]
    MensShirts,
    #[filter(icon = IconName::User)]
    MensShoes,
    #[filter(icon = IconName::Moon)]
    MensWatches,
    #[filter(icon = IconName::Settings)]
    MobileAccessories,
    #[filter(icon = IconName::ArrowUp)]
    Motorcycle,
    #[filter(icon = IconName::Star)]
    SkinCare,
    #[filter(icon = IconName::Settings)]
    Smartphones,
    #[filter(icon = IconName::Star)]
    SportsAccessories,
    #[filter(icon = IconName::Sun)]
    Sunglasses,
    #[filter(icon = IconName::Settings)]
    Tablets,
    #[filter(icon = IconName::User)]
    Tops,
    #[filter(icon = IconName::ArrowUp)]
    Vehicle,
    #[filter(icon = IconName::Search)]
    WomensBags,
    #[filter(icon = IconName::User)]
    WomensDresses,
    #[filter(icon = IconName::Star)]
    WomensJewellery,
    #[filter(icon = IconName::User)]
    WomensShoes,
    #[filter(icon = IconName::Moon)]
    WomensWatches,
}

/// A Product entry for the table - from DummyJSON API
#[derive(Clone, Debug, EsFluentKv, EsFluentThis, GpuiTable)]
#[fluent_this(origin, members)]
#[fluent_kv(keys = ["description", "label"])]
#[gpui_table(fluent = "label")]
#[gpui_table(load_more = "Self::load_more")]
#[gpui_table(load_more_threshold = 20)]
pub struct Product {
    /// Product ID from the API
    #[gpui_table(sortable, width = 50., resizable = false, movable = false)]
    pub id: u32,

    /// Product title - server-side searchable
    #[gpui_table(sortable, width = 200., filter(text()))]
    pub title: String,

    /// Product category - server-side filterable
    #[gpui_table(width = 120., filter(faceted()))]
    pub category: ProductCategory,

    /// Brand name
    #[gpui_table(width = 100.)]
    pub brand: String,

    /// Price in USD
    #[gpui_table(sortable, width = 80.)]
    pub price: f64,

    /// Discount percentage
    #[gpui_table(sortable, width = 80.)]
    pub discount_percentage: f64,

    /// Rating (0-5)
    #[gpui_table(sortable, width = 70.)]
    pub rating: f64,

    /// Stock count
    #[gpui_table(sortable, width = 70.)]
    pub stock: u32,
}

// ============================================================================
// DummyJSON API Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct DummyJsonProductsResponse {
    pub products: Vec<DummyJsonProduct>,
    pub total: u32,
    pub skip: u32,
    pub limit: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DummyJsonProduct {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub category: ProductCategory,
    pub price: f64,
    pub discount_percentage: f64,
    pub rating: f64,
    pub stock: u32,
    #[serde(default)]
    pub brand: Option<String>,
    pub sku: String,
    pub weight: f64,
}

impl Product {
    /// Convert from API response to our Product struct
    pub fn from_api(api: DummyJsonProduct) -> Self {
        Self {
            id: api.id,
            title: api.title,
            category: api.category,
            brand: api.brand.unwrap_or_default(),
            price: api.price,
            discount_percentage: api.discount_percentage,
            rating: api.rating,
            stock: api.stock,
        }
    }
}

// ============================================================================
// Table Delegate Implementation for API Loading
// ============================================================================

/// Tracks the current API fetch state
static API_SKIP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl ProductTableDelegate {
    /// Build URL with server-side filters
    fn build_api_url(skip: u32, limit: u32, filters: &ProductFilters) -> String {
        // If we have a category filter with exactly one category, use the category endpoint
        if filters.category.len() == 1 {
            let category_str = filters.category.iter().next().unwrap();
            // Convert PascalCase variant name to kebab-case for API
            let api_category = category_str.to_kebab_case();

            return format!(
                "https://dummyjson.com/products/category/{}?limit={}&skip={}",
                api_category, limit, skip
            );
        }

        // If we have a search query, use the search endpoint
        if !filters.title.is_empty() {
            return format!(
                "https://dummyjson.com/products/search?q={}&limit={}&skip={}",
                urlencoding::encode(&filters.title),
                limit,
                skip
            );
        }

        // Otherwise, use the base products endpoint
        format!(
            "https://dummyjson.com/products?limit={}&skip={}",
            limit, skip
        )
    }

    /// Log current filter state
    fn log_filter_state(filters: &ProductFilters) {
        let mut active_filters = Vec::new();

        if !filters.title.is_empty() {
            active_filters.push(format!("title=\"{}\"", filters.title));
        }
        if !filters.category.is_empty() {
            active_filters.push(format!("category={:?}", filters.category));
        }

        if active_filters.is_empty() {
            debug!("No active filters");
        } else {
            info!("Active filters: {}", active_filters.join(", "));
        }
    }

    /// Load more products from the API
    pub fn load_more(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
        if self.loading || self.eof {
            debug!(
                "Skipping load_more: loading={}, eof={}",
                self.loading, self.eof
            );
            return;
        }

        self.loading = true;
        cx.notify();

        let skip = API_SKIP.load(std::sync::atomic::Ordering::SeqCst);
        let limit = 20u32;

        // Clone filter values for use in async block
        let filters = self.filters.clone();

        // Log filter state
        Self::log_filter_state(&filters);

        let current_row_count = self.rows.len();

        // Build URL with server-side filters
        let url = Self::build_api_url(skip, limit, &filters);

        info!(
            "Fetching: skip={}, limit={}, current_rows={}",
            skip, limit, current_row_count
        );
        info!("GET {}", url);

        // Collect existing IDs to prevent duplicates
        let existing_ids: std::collections::HashSet<u32> = self.rows.iter().map(|p| p.id).collect();

        // Spawn the HTTP request on Tokio's runtime
        let tokio_task = Tokio::spawn(cx, async move {
            let result: Result<(Vec<Product>, u32), String> = async {
                let response = reqwest::get(&url)
                    .await
                    .map_err(|e| format!("Failed to fetch: {}", e))?;

                let data: DummyJsonProductsResponse = response
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse: {}", e))?;

                debug!(
                    "Response: total={}, skip={}, limit={}, returned={}",
                    data.total,
                    data.skip,
                    data.limit,
                    data.products.len()
                );

                let products: Vec<Product> =
                    data.products.into_iter().map(Product::from_api).collect();

                Ok((products, data.total))
            }
            .await;

            (result, limit)
        });

        // Spawn on GPUI to wait for the Tokio task and update UI
        cx.spawn(async move |view, cx| {
            match tokio_task.await {
                Ok((result, limit)) => {
                    _ = cx.update(|cx| {
                        view.update(cx, |table, cx| {
                            let delegate = table.delegate_mut();

                            match result {
                                Ok((new_products, total)) => {
                                    // Update the API skip offset
                                    let new_skip = API_SKIP
                                        .fetch_add(limit, std::sync::atomic::Ordering::SeqCst)
                                        + limit;

                                    let fetched_count = new_products.len();

                                    // Filter out duplicates
                                    let new_products: Vec<Product> = new_products
                                        .into_iter()
                                        .filter(|p| !existing_ids.contains(&p.id))
                                        .collect();

                                    info!(
                                        "Batch complete: fetched={}, new_skip={}",
                                        fetched_count, new_skip
                                    );

                                    delegate.rows.extend(new_products);

                                    info!(
                                        "Total rows: {}, api_total={}",
                                        delegate.rows.len(),
                                        total
                                    );

                                    // Check if we've reached the end
                                    if new_skip >= total || fetched_count == 0 {
                                        info!("Reached end: skip={} >= total={}", new_skip, total);
                                        delegate.eof = true;
                                    }
                                },
                                Err(e) => {
                                    warn!("API error: {}", e);
                                    delegate.eof = true;
                                },
                            }

                            delegate.loading = false;
                            cx.notify();
                        })
                        .unwrap();
                    });
                },
                Err(e) => {
                    warn!("Tokio task failed: {:?}", e);
                    _ = cx.update(|cx| {
                        view.update(cx, |table, cx| {
                            let delegate = table.delegate_mut();
                            delegate.loading = false;
                            delegate.eof = true;
                            cx.notify();
                        })
                        .unwrap();
                    });
                },
            }
        })
        .detach();
    }

    /// Reset and reload data (call when server-side filters change)
    pub fn reset_and_reload(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>) {
        info!("Resetting and reloading data (filters changed)");
        Self::log_filter_state(&self.filters);

        self.rows.clear();
        self.eof = false;
        self.loading = false;
        // Reset the API skip to start from the beginning
        API_SKIP.store(0, std::sync::atomic::Ordering::SeqCst);
        self.load_more(window, cx);
    }
}
