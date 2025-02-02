use c3p0_common::json::model::{IdType, VersionType};
use c3p0_common::Model;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ModelDto<DATA>
where
    DATA: Clone + serde::ser::Serialize + Send,
{
    pub id: IdType,
    pub version: VersionType,
    #[serde(bound(deserialize = "DATA: serde::Deserialize<'de>"))]
    pub data: DATA,
}

impl<DATA> From<Model<DATA>> for ModelDto<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
{
    fn from(model: Model<DATA>) -> Self {
        Self { id: model.id, version: model.version, data: model.data }
    }
}
