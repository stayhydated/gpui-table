//! Shared enums with all derives for SpacetimeDB, SeaORM, and gpui-table

use gpui_component::IconName;
use gpui_table::{Filterable, TableCell};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// ============================================================================
// SpacetimeDB Player Enums
// ============================================================================

/// Player status in the game
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    Hash,
    spacetimedb::SpacetimeType,
    es_fluent::EsFluent,
    Filterable,
    TableCell,
    strum::EnumIter,
)]
#[filter(fluent)]
pub enum PlayerStatus {
    #[default]
    #[filter(icon = IconName::CircleCheck)]
    Online,
    #[filter(icon = IconName::Moon)]
    Away,
    #[filter(icon = IconName::CircleX)]
    Offline,
    #[filter(icon = IconName::Star)]
    InGame,
}

/// Guild affiliation
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    Hash,
    spacetimedb::SpacetimeType,
    es_fluent::EsFluent,
    Filterable,
    TableCell,
    strum::EnumIter,
)]
#[filter(fluent)]
pub enum Guild {
    #[default]
    #[filter(icon = IconName::User)]
    None,
    #[filter(icon = IconName::ArrowUp)]
    Warriors,
    #[filter(icon = IconName::Star)]
    Mages,
    #[filter(icon = IconName::Settings)]
    Defenders,
    #[filter(icon = IconName::Sun)]
    Rogues,
    #[filter(icon = IconName::Check)]
    Healers,
}

// ============================================================================
// SeaORM Order Enums
// ============================================================================

/// Order status enum
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    es_fluent::EsFluent,
    Filterable,
    TableCell,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(50))")]
#[filter(fluent)]
pub enum OrderStatus {
    #[default]
    #[sea_orm(string_value = "pending")]
    #[filter(icon = IconName::Moon)]
    Pending,
    #[sea_orm(string_value = "confirmed")]
    #[filter(icon = IconName::Check)]
    Confirmed,
    #[sea_orm(string_value = "processing")]
    #[filter(icon = IconName::Settings)]
    Processing,
    #[sea_orm(string_value = "shipped")]
    #[filter(icon = IconName::ArrowUp)]
    Shipped,
    #[sea_orm(string_value = "delivered")]
    #[filter(icon = IconName::CircleCheck)]
    Delivered,
    #[sea_orm(string_value = "cancelled")]
    #[filter(icon = IconName::CircleX)]
    Cancelled,
    #[sea_orm(string_value = "refunded")]
    #[filter(icon = IconName::ArrowDown)]
    Refunded,
}

/// Shipping method enum
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    es_fluent::EsFluent,
    Filterable,
    TableCell,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(50))")]
#[filter(fluent)]
pub enum ShippingMethod {
    #[default]
    #[sea_orm(string_value = "standard")]
    #[filter(icon = IconName::Settings)]
    Standard,
    #[sea_orm(string_value = "express")]
    #[filter(icon = IconName::Star)]
    Express,
    #[sea_orm(string_value = "overnight")]
    #[filter(icon = IconName::ArrowUp)]
    Overnight,
    #[sea_orm(string_value = "pickup")]
    #[filter(icon = IconName::Search)]
    LocalPickup,
    #[sea_orm(string_value = "international")]
    #[filter(icon = IconName::Sun)]
    International,
}
