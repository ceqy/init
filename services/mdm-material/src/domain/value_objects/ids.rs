//! 强类型 ID 定义

use derive_more::{Display, From};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// 物料 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, From)]
#[display("{_0}")]
pub struct MaterialId(pub Uuid);

impl MaterialId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl FromStr for MaterialId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for MaterialId {
    fn default() -> Self {
        Self::new()
    }
}

/// 物料组 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, From)]
#[display("{_0}")]
pub struct MaterialGroupId(pub Uuid);

impl MaterialGroupId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl FromStr for MaterialGroupId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for MaterialGroupId {
    fn default() -> Self {
        Self::new()
    }
}

/// 物料类型 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, From)]
#[display("{_0}")]
pub struct MaterialTypeId(pub Uuid);

impl MaterialTypeId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl FromStr for MaterialTypeId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for MaterialTypeId {
    fn default() -> Self {
        Self::new()
    }
}
