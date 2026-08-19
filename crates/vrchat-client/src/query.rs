use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
pub enum QueryOrder {
    #[serde(rename = "ascending")]
    Ascending,
    #[serde(rename = "descending")]
    Descending,
}

impl QueryOrder {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ascending => "ascending",
            Self::Descending => "descending",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
pub enum ReleaseStatusFilter {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "hidden")]
    Hidden,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "public")]
    Public,
}

impl ReleaseStatusFilter {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Hidden => "hidden",
            Self::Private => "private",
            Self::Public => "public",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
pub enum AvatarListSort {
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "updated")]
    Updated,
    #[serde(rename = "order")]
    Order,
    #[serde(rename = "_created_at")]
    CreatedAt,
    #[serde(rename = "_updated_at")]
    UpdatedAt,
}

impl AvatarListSort {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Updated => "updated",
            Self::Order => "order",
            Self::CreatedAt => "_created_at",
            Self::UpdatedAt => "_updated_at",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
pub enum WorldSearchSort {
    #[serde(rename = "_created_at")]
    CreatedAt,
    #[serde(rename = "_updated_at")]
    UpdatedAt,
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "favorites")]
    Favorites,
    #[serde(rename = "heat")]
    Heat,
    #[serde(rename = "labsPublicationDate")]
    LabsPublicationDate,
    #[serde(rename = "magic")]
    Magic,
    #[serde(rename = "name")]
    Name,
    #[serde(rename = "order")]
    Order,
    #[serde(rename = "popularity")]
    Popularity,
    #[serde(rename = "publicationDate")]
    PublicationDate,
    #[serde(rename = "random")]
    Random,
    #[serde(rename = "relevance")]
    Relevance,
    #[serde(rename = "reportCount")]
    ReportCount,
    #[serde(rename = "reportScore")]
    ReportScore,
    #[serde(rename = "shuffle")]
    Shuffle,
    #[serde(rename = "trust")]
    Trust,
    #[serde(rename = "updated")]
    Updated,
}

impl WorldSearchSort {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CreatedAt => "_created_at",
            Self::UpdatedAt => "_updated_at",
            Self::Created => "created",
            Self::Favorites => "favorites",
            Self::Heat => "heat",
            Self::LabsPublicationDate => "labsPublicationDate",
            Self::Magic => "magic",
            Self::Name => "name",
            Self::Order => "order",
            Self::Popularity => "popularity",
            Self::PublicationDate => "publicationDate",
            Self::Random => "random",
            Self::Relevance => "relevance",
            Self::ReportCount => "reportCount",
            Self::ReportScore => "reportScore",
            Self::Shuffle => "shuffle",
            Self::Trust => "trust",
            Self::Updated => "updated",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
pub enum UserSearchCustomField {
    #[serde(rename = "bio")]
    Bio,
    #[serde(rename = "displayName")]
    DisplayName,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
pub enum UserSearchSort {
    #[serde(rename = "_created_at")]
    CreatedAt,
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "last_login")]
    LastLogin,
    #[serde(rename = "nuisanceFactor")]
    NuisanceFactor,
    #[serde(rename = "relevance")]
    Relevance,
}

pub(crate) fn serialize_query<T: Serialize>(query: &T) -> HashMap<String, Value> {
    let Value::Object(query) =
        serde_json::to_value(query).expect("query DTO serialization must succeed")
    else {
        panic!("query DTO must serialize as an object");
    };
    query.into_iter().collect()
}
