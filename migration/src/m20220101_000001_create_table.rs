use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Account::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Account::Email).string().primary_key())
                    .col(ColumnDef::new(Account::Salt).uuid().not_null())
                    .col(ColumnDef::new(Account::PasswordHash).binary().binary_len(24).not_null())
                    .col(ColumnDef::new(Account::IsAdministrator).boolean().not_null())
                    .col(ColumnDef::new(Account::IsEditor).boolean().not_null())
                    .col(
                        ColumnDef::new(Account::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Account::UpdatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Account::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Account {
    Table,
    Email,
    Salt,
    PasswordHash,
    IsAdministrator,
    IsEditor,
    CreatedAt,
    UpdatedAt,
}
