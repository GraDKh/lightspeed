use crate::model::{Repository, RepositoryFile};
use crate::repository::db::{
    DBFileStoreBinaryRepository, DBFileStoreRepositoryManager, FileStoreDataRepository,
};
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use log::*;
use std::sync::Arc;
use crate::service::file_store::FileStoreService;

#[derive(Clone)]
pub struct FileMigratorService<RepoManager: DBFileStoreRepositoryManager> {
    c3p0: RepoManager::C3P0,
    db_binary_repo: RepoManager::FileStoreBinaryRepo,
    db_data_repo: RepoManager::FileStoreDataRepo,
    file_store: Arc<FileStoreService<RepoManager>>,
}

impl<RepoManager: DBFileStoreRepositoryManager> FileMigratorService<RepoManager> {
    pub fn new(repo_manager: &RepoManager, file_store: Arc<FileStoreService<RepoManager>>) -> Self {
        FileMigratorService {
            c3p0: repo_manager.c3p0().clone(),
            db_binary_repo: repo_manager.file_store_binary_repo(),
            db_data_repo: repo_manager.file_store_data_repo(),
            file_store,
        }
    }


    pub async fn migrate_all_files(
        &self,
        source_repository: &Repository,
        dest_repository: &Repository,
    ) -> Result<usize, LightSpeedError> {

        info!("Start migration of all files from {:?} to {:?}", source_repository, dest_repository);

        self.c3p0
            .transaction(|mut conn| async move {
                let conn = &mut conn;


                let offset = 0;
                let max = 100;
                let order = OrderBy::Asc;
                let mut more_files = true;
                let mut total_migrated = 0;

                while more_files {
                    let files = self.file_store.read_all_file_data_by_repository_with_conn(conn, source_repository, offset, max, &order).await?;
                    more_files = !files.is_empty();

                    for mut file in files {

                        debug!("migrate file [{}] from [{:?}] to [{:?}]", file.data.filename, file.data.repository, dest_repository);

                        match file.data.repository {
                            RepositoryFile::DB {repository_name: _source_repository_name, ..} => {
                                match dest_repository {
                                    Repository::DB {repository_name: _dest_repository_name} => {
                                        return Err(LightSpeedError::InternalServerError {
                                            message: "Migration from DB to DB is not implemented".to_owned()
                                        });
                                    },
                                    Repository::FS {repository_name: _dest_repository_name} => {
                                        return Err(LightSpeedError::InternalServerError {
                                            message: "Migration from DB to FS is not implemented".to_owned()
                                        });
                                    }
                                }
                            },
                            RepositoryFile::FS {repository_name: source_repository_name, file_path} => {
                                match dest_repository {
                                    Repository::DB {repository_name: dest_repository_name} => {
                                        let source_file_content = self.file_store.read_file_content_from_fs(&file_path, &source_repository_name).await?;
                                        self.db_binary_repo.save_file(conn, &dest_repository_name, &file_path, &source_file_content).await?;
                                        file.data.repository = RepositoryFile::DB {
                                            file_path,
                                            repository_name: dest_repository_name.to_owned()
                                        };
                                        self.db_data_repo.update(conn, file).await?;
                                    },
                                    Repository::FS {repository_name: _dest_repository_name} => {
                                        return Err(LightSpeedError::InternalServerError {
                                            message: "Migration from FS to FS is not implemented".to_owned()
                                        });
                                    }
                                }
                            }
                        }
                        total_migrated += 1;
                    }

                }

                info!("Completed migration of {} files from {:?} to {:?}", total_migrated, source_repository, dest_repository);
                Ok(total_migrated)

            })
            .await

    }

}
