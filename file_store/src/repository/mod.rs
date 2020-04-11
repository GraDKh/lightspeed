use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use std::path::Path;

pub mod filesystem;
pub mod pg;

#[async_trait::async_trait(?Send)]
pub trait FileStoreRepositoryManager: Clone + Send + Sync {
    type Conn: SqlConnectionAsync;
    type C3P0: C3p0PoolAsync<CONN = Self::Conn>;
    type FileStoreBinaryRepo: FileStoreBinaryRepository<Conn = Self::Conn>;
    type FileStoreDataRepo: FileStoreDataRepository<Conn = Self::Conn>;

    fn c3p0(&self) -> &Self::C3P0;
    async fn start(&self) -> Result<(), LightSpeedError>;

    fn file_store_data_repo(&self) -> Self::FileStoreDataRepo;
    fn file_store_binary_repo(&self) -> Self::FileStoreBinaryRepo;
}

#[async_trait::async_trait]
pub trait FileStoreBinaryRepository: Clone + Send + Sync {
    type Conn: SqlConnectionAsync;

    async fn read_file<W: tokio::io::AsyncWrite + Unpin + Send>(&self, conn: &mut Self::Conn, file_name: &str, output: &mut W) -> Result<u64, LightSpeedError>;

    async fn save_file(&self, conn: &mut Self::Conn, source_path: &str, file_name: &str) -> Result<(), LightSpeedError>;

    async fn delete_by_filename(&self, conn: &mut Self::Conn, file_name: &str) -> Result<(), LightSpeedError>;


}

#[async_trait::async_trait]
pub trait FileStoreDataRepository: Clone + Send + Sync {
    type Conn: SqlConnectionAsync;
}
