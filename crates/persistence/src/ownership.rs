use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::common::{normalize_text, row_i64, DbWriteTarget, ParamsBuilder};
use crate::database::DatabaseService;
use crate::Error;

pub(crate) const COL_OWNER_ID: &str = "owner_id";

#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Deserialize,
    Serialize,
    specta::Type,
)]
#[serde(transparent)]
pub struct OwnerId(String);

impl OwnerId {
    pub fn new(owner_user_id: impl Into<String>) -> Self {
        Self(owner_user_id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for OwnerId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl From<&OwnerId> for Value {
    fn from(owner_user_id: &OwnerId) -> Self {
        Value::String(owner_user_id.0.clone())
    }
}

impl From<OwnerId> for Value {
    fn from(owner_user_id: OwnerId) -> Self {
        Value::String(owner_user_id.0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct OwnerRowId(i64);

impl OwnerRowId {
    pub(crate) const UNASSIGNED: Self = Self(0);

    pub(crate) fn value(self) -> i64 {
        self.0
    }
}

impl From<OwnerRowId> for Value {
    fn from(owner_id: OwnerRowId) -> Self {
        Value::from(owner_id.value())
    }
}

pub(crate) fn ensure_owner_table(db: &DatabaseService) -> Result<(), Error> {
    ensure_owner_table_on(db)
}

pub(crate) fn ensure_owner_table_on(target: &impl DbWriteTarget) -> Result<(), Error> {
    target.execute_non_query(
        "CREATE TABLE IF NOT EXISTS owners (id INTEGER PRIMARY KEY AUTOINCREMENT, user_id TEXT NOT NULL UNIQUE)",
        &Default::default(),
    )?;
    Ok(())
}

pub(crate) fn owner_id_get(
    db: &DatabaseService,
    owner_user_id: &OwnerId,
) -> Result<Option<OwnerRowId>, Error> {
    owner_row_id_lookup(db, &normalize_text(owner_user_id.as_str()))
}

pub(crate) fn owner_id_get_or_insert(
    db: &DatabaseService,
    owner_user_id: &OwnerId,
) -> Result<OwnerRowId, Error> {
    let owner_user_id = normalize_text(owner_user_id.as_str());
    if owner_user_id.is_empty() {
        return Ok(OwnerRowId::UNASSIGNED);
    }
    ensure_owner_table(db)?;
    db.execute_non_query(
        "INSERT OR IGNORE INTO owners (user_id) VALUES (@user_id)",
        &ParamsBuilder::new()
            .set("user_id", owner_user_id.clone())
            .build(),
    )?;
    owner_row_id_lookup(db, &owner_user_id)?.ok_or_else(|| {
        Error::Database("Owner dictionary row was not available after insertion.".into())
    })
}

pub(crate) fn owner_id_for_filter(
    db: &DatabaseService,
    owner_user_id: &OwnerId,
) -> Result<OwnerRowId, Error> {
    Ok(owner_id_get(db, owner_user_id)?.unwrap_or(OwnerRowId::UNASSIGNED))
}

fn owner_row_id_lookup(
    db: &DatabaseService,
    normalized_owner_user_id: &str,
) -> Result<Option<OwnerRowId>, Error> {
    if normalized_owner_user_id.is_empty() {
        return Ok(None);
    }
    ensure_owner_table(db)?;
    Ok(db
        .execute(
            "SELECT id FROM owners WHERE user_id = @user_id LIMIT 1",
            &ParamsBuilder::new()
                .set("user_id", normalized_owner_user_id)
                .build(),
        )?
        .first()
        .map(|row| OwnerRowId(row_i64(row, 0))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db(name: &str) -> DatabaseService {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "vrcx-0-owner-{name}-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        DatabaseService::new(&dir.join("VRCX-0.sqlite3")).unwrap()
    }

    #[test]
    fn owner_dictionary_is_idempotent_and_read_does_not_insert() {
        let db = test_db("dictionary");

        let missing = OwnerId::new("usr_missing");
        assert_eq!(owner_id_get(&db, &missing).unwrap(), None);
        let first = owner_id_get_or_insert(&db, &OwnerId::new(" usr_owner ")).unwrap();
        let second = owner_id_get_or_insert(&db, &OwnerId::new("usr_owner")).unwrap();

        assert!(first > OwnerRowId::UNASSIGNED);
        assert_eq!(first, second);
        assert_eq!(owner_id_get(&db, &missing).unwrap(), None);
        let rows = db
            .execute(
                "SELECT user_id FROM owners ORDER BY id",
                &Default::default(),
            )
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0].as_str(), Some("usr_owner"));
    }
}
