use crate::model::content::{ContentData, ContentModel};
use crate::model::project::{ProjectData, ProjectModel};
use crate::model::schema::{SchemaData, SchemaModel};
use c3p0::{C3p0Pool, NewModel};
use lightspeed_core::error::LightSpeedError;

pub mod pg;

pub trait CmsRepositoryManager: Clone {
    type Conn;
    type C3P0: C3p0Pool<CONN = Self::Conn>;
    type ContentRepo: ContentRepository<Conn = Self::Conn>;
    type ProjectRepo: ProjectRepository<Conn = Self::Conn>;
    type SchemaRepo: SchemaRepository<Conn = Self::Conn>;

    fn c3p0(&self) -> &Self::C3P0;
    fn start(&self) -> Result<(), LightSpeedError>;

    fn content_repo(&self, qualified_table_name: &str) -> Self::ContentRepo;
    fn project_repo(&self) -> Self::ProjectRepo;
    fn schema_repo(&self) -> Self::SchemaRepo;
}

pub trait ProjectRepository: Clone {
    type Conn;

    fn fetch_by_id(&self, conn: &mut Self::Conn, id: i64) -> Result<ProjectModel, LightSpeedError>;

    fn exists_by_name(&self, conn: &mut Self::Conn, name: &str) -> Result<bool, LightSpeedError>;

    fn save(
        &self,
        conn: &mut Self::Conn,
        model: NewModel<ProjectData>,
    ) -> Result<ProjectModel, LightSpeedError>;

    fn update(
        &self,
        conn: &mut Self::Conn,
        model: ProjectModel,
    ) -> Result<ProjectModel, LightSpeedError>;

    fn delete(&self, conn: &mut Self::Conn, model: ProjectModel) -> Result<ProjectModel, LightSpeedError>;
}

pub trait SchemaRepository: Clone {
    type Conn;

    fn fetch_by_id(&self, conn: &mut Self::Conn, id: i64) -> Result<SchemaModel, LightSpeedError>;

    fn exists_by_name_and_project_id(
        &self,
        conn: &mut Self::Conn,
        name: &str,
        project_id: i64,
    ) -> Result<bool, LightSpeedError>;

    fn save(
        &self,
        conn: &mut Self::Conn,
        model: NewModel<SchemaData>,
    ) -> Result<SchemaModel, LightSpeedError>;

    fn update(&self, conn: &mut Self::Conn, model: SchemaModel)
              -> Result<SchemaModel, LightSpeedError>;

    fn delete(&self, conn: &mut Self::Conn, model: SchemaModel) -> Result<SchemaModel, LightSpeedError>;

    fn delete_by_project_id(
        &self,
        conn: &mut Self::Conn,
        project_id: i64,
    ) -> Result<u64, LightSpeedError>;
}

pub trait ContentRepository: Clone {
    type Conn;

    fn create_table(&self, conn: &mut Self::Conn) -> Result<(), LightSpeedError>;

    fn drop_table(&self, conn: &mut Self::Conn) -> Result<(), LightSpeedError>;

    fn count_all(&self, conn: &mut Self::Conn) -> Result<u64, LightSpeedError>;

    fn count_all_by_field_value(&self, conn: &mut Self::Conn, field_name: &str, field_value: &str) -> Result<u64, LightSpeedError>;

    fn create_unique_constraint(
        &self,
        conn: &mut Self::Conn,
        index_name: &str,
        field_name: &str,
    ) -> Result<(), LightSpeedError>;

    fn drop_unique_constraint(
        &self,
        conn: &mut Self::Conn,
        index_name: &str,
    ) -> Result<(), LightSpeedError>;

    fn fetch_by_id(&self, conn: &mut Self::Conn, id: i64) -> Result<ContentModel, LightSpeedError>;

    fn save(
        &self,
        conn: &mut Self::Conn,
        model: NewModel<ContentData>,
    ) -> Result<ContentModel, LightSpeedError>;

    fn update(
        &self,
        conn: &mut Self::Conn,
        model: ContentModel,
    ) -> Result<ContentModel, LightSpeedError>;

    fn delete(&self, conn: &mut Self::Conn, model: ContentModel) -> Result<ContentModel, LightSpeedError>;
}
