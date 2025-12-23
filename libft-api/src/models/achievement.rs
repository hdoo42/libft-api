use crate::models::prelude::*;
use rvstruct::ValueStruct;
use serde::{Deserialize, Serialize};

// use crate::models::prelude::*;

// FtAchievement and its field structs
//

// Assuming FtAchievementId is not defined elsewhere:
#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Clone, Serialize, Deserialize, ValueStruct)]
pub struct FtAchievementId(pub u64);

#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Clone, Serialize, Deserialize, ValueStruct)]
pub struct FtAchievementName(pub String);

#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Clone, Serialize, Deserialize, ValueStruct)]
pub struct FtAchievementDescription(pub String);

#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Clone, Serialize, Deserialize, ValueStruct)]
pub struct FtAchievementTier(pub String);

#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Clone, Serialize, Deserialize, ValueStruct)]
pub struct FtAchievementKind(pub String);

#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Clone, Serialize, Deserialize, ValueStruct)]
pub struct FtAchievementImage(pub String);

#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Clone, Serialize, Deserialize, ValueStruct)]
pub struct FtAchievementNbrOfSuccess(pub u64);

#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Clone, Serialize, Deserialize, ValueStruct)]
pub struct FtAchievementUsersUrl(pub FtUrl);

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct FtAchievement {
    pub id: Option<FtAchievementId>,
    pub name: Option<FtAchievementName>,
    pub description: Option<FtAchievementDescription>,
    pub tier: Option<FtAchievementTier>,
    pub kind: Option<FtAchievementKind>,
    pub visible: Option<bool>,
    pub image: Option<FtAchievementImage>,
    pub nbr_of_success: Option<FtAchievementNbrOfSuccess>,
    pub users_url: Option<FtAchievementUsersUrl>,
}
