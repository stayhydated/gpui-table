use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Orders::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Orders::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Orders::CustomerName)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Orders::CustomerEmail)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Orders::Status)
                            .string_len(50)
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(Orders::TotalAmount)
                            .decimal_len(10, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Orders::ItemCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Orders::ShippingMethod)
                            .string_len(50)
                            .not_null()
                            .default("standard"),
                    )
                    .col(
                        ColumnDef::new(Orders::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Orders::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on status for filtering
        manager
            .create_index(
                Index::create()
                    .name("idx_orders_status")
                    .table(Orders::Table)
                    .col(Orders::Status)
                    .to_owned(),
            )
            .await?;

        // Create index on created_at for sorting/filtering
        manager
            .create_index(
                Index::create()
                    .name("idx_orders_created_at")
                    .table(Orders::Table)
                    .col(Orders::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // Create index on customer_name for text search
        manager
            .create_index(
                Index::create()
                    .name("idx_orders_customer_name")
                    .table(Orders::Table)
                    .col(Orders::CustomerName)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Orders::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Orders {
    Table,
    Id,
    CustomerName,
    CustomerEmail,
    Status,
    TotalAmount,
    ItemCount,
    ShippingMethod,
    CreatedAt,
    UpdatedAt,
}
