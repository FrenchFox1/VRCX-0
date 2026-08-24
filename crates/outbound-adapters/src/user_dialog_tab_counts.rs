use std::sync::Arc;

use url::Url;
use uuid::Uuid;
use vrcx_0_application::social::{
    AvatarProviderConfig, AvatarReleaseStatus, UserDialogExternalFuture, UserDialogTabCountsSource,
};
use vrcx_0_application_core::{vrchat_api::VrchatApiRequest, Result, WebClient};
use vrcx_0_integrations::external_api::{self, ExternalApiScope};
use vrcx_0_persistence::{config, DatabaseService};
use vrcx_0_vrchat_client::{
    avatars::{avatar_list_by_user_get_input, AvatarListByUserGetInput},
    favorites::{favorite_groups_get_input, favorite_worlds_get_input},
    groups::user_groups_get_input,
    query::{AvatarListSort, QueryOrder, ReleaseStatusFilter, WorldSearchSort},
    users::user_mutual_counts_get_input,
    worlds::world_list_by_user_get_input,
};

const DEFAULT_AVATAR_PROVIDER: &str = "https://api.avtrdb.com/v3/avatar/search/vrcx";
const AVATAR_PROVIDER_ENABLED_KEY: &str = "avatarRemoteDatabase";
const AVATAR_PROVIDER_LIST_KEY: &str = "VRCX_avatarRemoteDatabaseProviderList";
const AVATAR_PROVIDER_SELECTED_KEY: &str = "VRCX_avatarRemoteDatabaseProvider";
const VRCX_ID_KEY: &str = "id";

pub struct LocalUserDialogTabCountsSource {
    db: Arc<DatabaseService>,
    web: Arc<WebClient>,
}

impl LocalUserDialogTabCountsSource {
    pub fn new(db: Arc<DatabaseService>, web: Arc<WebClient>) -> Self {
        Self { db, web }
    }
}

impl UserDialogTabCountsSource for LocalUserDialogTabCountsSource {
    fn avatar_provider_config(&self) -> Result<AvatarProviderConfig> {
        Ok(AvatarProviderConfig {
            enabled: config::get_bool(&self.db, AVATAR_PROVIDER_ENABLED_KEY, true)
                .map_err(crate::map_persistence_error)?,
            providers: config::get_json(
                &self.db,
                AVATAR_PROVIDER_LIST_KEY,
                serde_json::json!([DEFAULT_AVATAR_PROVIDER]),
            )
            .map_err(crate::map_persistence_error)?
            .into(),
            selected: config::get_string(&self.db, AVATAR_PROVIDER_SELECTED_KEY, "")
                .map_err(crate::map_persistence_error)?,
        })
    }

    fn mutual_friends(&self, endpoint: String, user_id: String) -> Result<VrchatApiRequest> {
        Ok(user_mutual_counts_get_input(endpoint, user_id)?.1)
    }

    fn groups(&self, endpoint: String, user_id: String) -> Result<VrchatApiRequest> {
        Ok(user_groups_get_input(endpoint, user_id)?.1)
    }

    fn worlds(
        &self,
        endpoint: String,
        user_id: String,
        n: i32,
        offset: i32,
        release_status: AvatarReleaseStatus,
    ) -> Result<VrchatApiRequest> {
        Ok(world_list_by_user_get_input(
            endpoint,
            user_id,
            n,
            offset,
            WorldSearchSort::Updated,
            QueryOrder::Descending,
            release_status_filter(release_status),
        )?
        .1)
    }

    fn favorite_worlds(
        &self,
        endpoint: String,
        user_id: String,
        group_name: String,
        n: i32,
        offset: i32,
    ) -> Result<VrchatApiRequest> {
        Ok(favorite_worlds_get_input(
            endpoint,
            n,
            offset,
            user_id.clone(),
            user_id,
            group_name,
        ))
    }

    fn favorite_groups(
        &self,
        endpoint: String,
        user_id: String,
        n: i32,
        offset: i32,
    ) -> Result<VrchatApiRequest> {
        Ok(favorite_groups_get_input(endpoint, n, offset, user_id))
    }

    fn my_avatars(&self, endpoint: String, n: i32, offset: i32) -> Result<VrchatApiRequest> {
        Ok(avatar_list_by_user_get_input(AvatarListByUserGetInput {
            endpoint,
            user_id: String::new(),
            user: "me".into(),
            n,
            offset,
            sort: AvatarListSort::Updated,
            order: QueryOrder::Descending,
            release_status: ReleaseStatusFilter::All,
        })?
        .1)
    }

    fn external_avatar_search<'a>(
        &'a self,
        provider: &'a str,
        target_user_id: &'a str,
    ) -> UserDialogExternalFuture<'a> {
        Box::pin(async move {
            let mut url = Url::parse(provider).map_err(|error| {
                crate::Error::Custom(format!("Invalid avatar provider URL: {error}"))
            })?;
            let retained_query = url
                .query_pairs()
                .filter(|(key, _)| key != "search" && key != "n")
                .map(|(key, value)| (key.into_owned(), value.into_owned()))
                .collect::<Vec<_>>();
            url.set_query(None);
            {
                let mut query = url.query_pairs_mut();
                query.extend_pairs(retained_query);
                query.append_pair("search", target_user_id);
                query.append_pair("n", "5000");
            }

            let mut vrcx_id = config::get_string(&self.db, VRCX_ID_KEY, "")
                .map_err(crate::map_persistence_error)?
                .trim()
                .to_string();
            if vrcx_id.is_empty() {
                vrcx_id = Uuid::new_v4().to_string();
                config::set_string(&self.db, VRCX_ID_KEY, &vrcx_id)
                    .map_err(crate::map_persistence_error)?;
            }
            let input = external_api::avatar_search_get_input(url.as_str(), &vrcx_id);
            let request =
                external_api::build_web_execute_request(input, ExternalApiScope::AvatarSearch)
                    .map_err(|error| crate::Error::Custom(error.to_string()))?;
            self.web.execute_external(request).await
        })
    }
}

fn release_status_filter(status: AvatarReleaseStatus) -> ReleaseStatusFilter {
    match status {
        AvatarReleaseStatus::All => ReleaseStatusFilter::All,
        AvatarReleaseStatus::Hidden => ReleaseStatusFilter::Hidden,
        AvatarReleaseStatus::Private => ReleaseStatusFilter::Private,
        AvatarReleaseStatus::Public => ReleaseStatusFilter::Public,
    }
}
