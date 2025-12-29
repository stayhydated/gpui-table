//! SeaORM Example - Order Management with PostgreSQL

use chrono::{DateTime, Utc};
use es_fluent::{EsFluentKv, EsFluentThis};
use gpui::{Context, Window};
use gpui_component::table::TableState;
use gpui_table::GpuiTable;
use gpui_tokio::Tokio;
use log::info;
use rust_decimal::Decimal;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

// Re-export the generated FilterValues type
pub use ModelFilterValues as SeaormOrderFilterValues;

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Use enums from the local enums module
use super::enums::{OrderStatus, ShippingMethod};

// ============================================================================
// SeaORM Entity + GPUI Table Definition (combined)
// ============================================================================

/// Order entity - maps to `orders` table in PostgreSQL and gpui-table display
///
/// Note: SeaORM requires the struct to be named `Model`, so we use a type alias
/// `SeaormOrder` for external use.
#[derive(
    Clone,
    Debug,
    PartialEq,
    DeriveEntityModel,
    Serialize,
    Deserialize,
    EsFluentKv,
    EsFluentThis,
    GpuiTable,
)]
#[sea_orm(table_name = "orders")]
#[fluent_this(origin, members)]
#[fluent_kv(keys = ["description", "label"])]
#[gpui_table(fluent = "label")]
#[gpui_table(load_more = "Self::load_more")]
#[gpui_table(load_more_threshold = 20)]
pub struct Model {
    #[sea_orm(primary_key)]
    #[gpui_table(sortable, width = 70., resizable = false, movable = false)]
    pub id: i64,

    #[sea_orm(column_type = "String(StringLen::N(255))")]
    #[gpui_table(sortable, width = 150., filter(text()))]
    pub customer_name: String,

    #[sea_orm(column_type = "String(StringLen::N(255))")]
    #[gpui_table(width = 180., filter(text()))]
    pub customer_email: String,

    #[gpui_table(width = 100., filter(faceted()))]
    pub status: OrderStatus,

    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    #[gpui_table(sortable, width = 100., filter(number_range(min = 0., max = 10000.)))]
    pub total_amount: Decimal,

    #[gpui_table(sortable, width = 70.)]
    pub item_count: i32,

    #[gpui_table(width = 100., filter(faceted()))]
    pub shipping_method: ShippingMethod,

    #[gpui_table(sortable, width = 180., filter(date_range()))]
    pub created_at: DateTime<Utc>,

    #[gpui_table(skip)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// ============================================================================
// Database Connection
// ============================================================================

static DB_CONNECTION: std::sync::OnceLock<Arc<DatabaseConnection>> = std::sync::OnceLock::new();
static CURRENT_PAGE: AtomicU64 = AtomicU64::new(0);

/// Initialize the database connection (call once at startup)
pub async fn init_database(database_url: &str) -> Result<(), sea_orm::DbErr> {
    let db = sea_orm::Database::connect(database_url).await?;
    DB_CONNECTION
        .set(Arc::new(db))
        .map_err(|_| sea_orm::DbErr::Custom("Database already initialized".into()))?;
    Ok(())
}

/// Get the database connection
pub fn get_database() -> Option<Arc<DatabaseConnection>> {
    DB_CONNECTION.get().cloned()
}

// ============================================================================
// Table Delegate Implementation
// ============================================================================

impl ModelTableDelegate {
    /// Fetch orders from PostgreSQL with filters
    async fn fetch_orders(
        db: &DatabaseConnection,
        filters: &ModelFilterValues,
        page: u64,
        page_size: u64,
    ) -> Result<Vec<Model>, sea_orm::DbErr> {
        let mut query = Entity::find();

        // Apply customer name filter
        if filters.customer_name.is_active() {
            query = query.filter(Column::CustomerName.contains(filters.customer_name.as_str()));
        }

        // Apply amount range filters
        if let Some(min) = filters.total_amount.min() {
            query =
                query.filter(Column::TotalAmount.gte(Decimal::try_from(*min).unwrap_or_default()));
        }
        if let Some(max) = filters.total_amount.max() {
            query =
                query.filter(Column::TotalAmount.lte(Decimal::try_from(*max).unwrap_or_default()));
        }

        // Apply status filter
        if filters.status.is_active() {
            let statuses: Vec<OrderStatus> = filters.status.iter().cloned().collect();
            query = query.filter(Column::Status.is_in(statuses));
        }

        // Apply shipping method filter
        if filters.shipping_method.is_active() {
            let methods: Vec<ShippingMethod> = filters.shipping_method.iter().cloned().collect();
            query = query.filter(Column::ShippingMethod.is_in(methods));
        }

        // Order by created_at descending and paginate
        query
            .order_by_desc(Column::CreatedAt)
            .paginate(db, page_size)
            .fetch_page(page)
            .await
    }

    pub fn load_more_with_filters(
        &mut self,
        filters: ModelFilterValues,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) {
        if self.loading || self.eof {
            return;
        }

        let Some(db) = get_database() else {
            info!("SeaORM: No database connection available");
            return;
        };

        self.loading = true;
        cx.notify();

        let page = CURRENT_PAGE.load(Ordering::SeqCst);
        let page_size = 30u64;

        info!(
            "SeaORM: Fetching page {} from PostgreSQL (filters: customer='{}', statuses={:?})",
            page, filters.customer_name, filters.status
        );

        let tokio_task = Tokio::spawn(cx, async move {
            Self::fetch_orders(&db, &filters, page, page_size).await
        });

        cx.spawn(async move |view, cx| match tokio_task.await {
            Ok(Ok(orders)) => {
                _ = cx.update(|cx| {
                    view.update(cx, |table, cx| {
                        let delegate = table.delegate_mut();
                        CURRENT_PAGE.fetch_add(1, Ordering::SeqCst);

                        let fetched_count = orders.len();
                        info!("SeaORM: Received {} orders from PostgreSQL", fetched_count);

                        delegate.rows.extend(orders);

                        if fetched_count < 30 {
                            delegate.eof = true;
                        }

                        delegate.loading = false;
                        cx.notify();
                    })
                    .ok();
                });
            },
            Ok(Err(e)) => {
                info!("SeaORM query error: {:?}", e);
                _ = cx.update(|cx| {
                    view.update(cx, |table, cx| {
                        let delegate = table.delegate_mut();
                        delegate.loading = false;
                        delegate.eof = true;
                        cx.notify();
                    })
                    .ok();
                });
            },
            Err(e) => {
                info!("SeaORM task error: {:?}", e);
            },
        })
        .detach();
    }

    pub fn load_more(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>) {
        self.load_more_with_filters(ModelFilterValues::default(), window, cx);
    }

    pub fn reset_and_reload_with_filters(
        &mut self,
        filters: ModelFilterValues,
        window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) {
        info!("SeaORM: Resetting pagination and reloading with new filters");
        self.rows.clear();
        self.eof = false;
        self.loading = false;
        CURRENT_PAGE.store(0, Ordering::SeqCst);

        self.load_more_with_filters(filters, window, cx);
    }
}
