use crate::repository::pg::pg_auth_account::PgAuthAccountRepository;
use crate::repository::pg::pg_token::PgTokenRepository;
use crate::repository::AuthRepositoryManager;
use c3p0::pg_async::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;

pub mod pg_auth_account;
pub mod pg_token;

const MIGRATIONS: include_dir::Dir = include_dir::include_dir!("./src_resources/db/pg/migrations");

#[derive(Clone)]
pub struct PgAuthRepositoryManager {
    c3p0: PgC3p0PoolAsync,
}

impl PgAuthRepositoryManager {
    pub fn new(c3p0: PgC3p0PoolAsync) -> Self {
        Self { c3p0 }
    }
}

#[async_trait::async_trait]
impl AuthRepositoryManager for PgAuthRepositoryManager {
    type Conn = PgConnectionAsync;
    type C3P0 = PgC3p0PoolAsync;
    type AuthAccountRepo = PgAuthAccountRepository;
    type TokenRepo = PgTokenRepository;

    fn c3p0(&self) -> &PgC3p0PoolAsync {
        &self.c3p0
    }

    async fn start(&self) -> Result<(), LightSpeedError> {
        let migrate_table_name = format!("LS_AUTH_{}", C3P0_MIGRATE_TABLE_DEFAULT);

        let migrate = C3p0MigrateBuilder::new(self.c3p0().clone())
            .with_table_name(migrate_table_name)
            .with_migrations(from_embed(&MIGRATIONS).map_err(|err| {
                LightSpeedError::ModuleStartError {
                    message: format!(
                        "PgAuthRepositoryManager - failed to read db migrations: {}",
                        err
                    ),
                }
            })?)
            .build();

        migrate
            .migrate()
            .await
            .map_err(|err| LightSpeedError::ModuleStartError {
                message: format!("PgAuthRepositoryManager - db migration failed: {}", err),
            })
    }

    fn auth_account_repo(&self) -> Self::AuthAccountRepo {
        PgAuthAccountRepository::default()
    }

    fn token_repo(&self) -> Self::TokenRepo {
        PgTokenRepository::default()
    }
}
