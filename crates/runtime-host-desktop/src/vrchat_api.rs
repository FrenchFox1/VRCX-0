use std::sync::Arc;

use vrcx_0_application::remote::{VrchatApiFuture, VrchatApiPort, VrchatApiRuntime};
use vrcx_0_application_core::vrchat_api::{self, VrchatApiRequest, VrchatScope};
use vrcx_0_application_core::{
    RemoteMutationGate, RuntimeAuthScope, RuntimeDiagnostics, RuntimeSyncEngine, WebClient,
};

pub mod protocol {
    pub mod auth {
        pub use vrcx_0_vrchat_client::auth::*;
    }
    pub mod avatars {
        pub use vrcx_0_vrchat_client::avatars::*;
    }
    pub mod favorites {
        pub use vrcx_0_vrchat_client::favorites::*;
    }
    pub mod friends {
        pub use vrcx_0_vrchat_client::friends::*;
    }
    pub mod instances {
        pub use vrcx_0_vrchat_client::instances::*;
    }
    pub mod media {
        pub use vrcx_0_vrchat_client::media::*;
    }
    pub mod notifications {
        pub use vrcx_0_vrchat_client::notifications::*;
    }
    pub mod query {
        pub use vrcx_0_vrchat_client::query::*;
    }
    pub mod search {
        pub use vrcx_0_vrchat_client::search::*;
    }
    pub mod tools {
        pub use vrcx_0_vrchat_client::tools::*;
    }
    pub mod users {
        pub use vrcx_0_vrchat_client::users::*;
    }
    pub mod worlds {
        pub use vrcx_0_vrchat_client::worlds::*;
    }
}

pub struct VrchatApiRuntimeDeps {
    pub auth_scope: RuntimeAuthScope,
    pub remote_mutations: Arc<RemoteMutationGate>,
    pub web: Arc<WebClient>,
    pub diagnostics: RuntimeDiagnostics,
    pub sync: RuntimeSyncEngine,
}

pub fn build_vrchat_api_runtime(deps: VrchatApiRuntimeDeps) -> VrchatApiRuntime {
    VrchatApiRuntime::new(
        deps.auth_scope,
        deps.remote_mutations,
        Arc::new(DesktopVrchatApiPort {
            web: deps.web,
            diagnostics: deps.diagnostics,
            sync: deps.sync,
        }),
    )
}

struct DesktopVrchatApiPort {
    web: Arc<WebClient>,
    diagnostics: RuntimeDiagnostics,
    sync: RuntimeSyncEngine,
}

impl VrchatApiPort for DesktopVrchatApiPort {
    fn execute(
        &self,
        command: String,
        detail: String,
        input: VrchatApiRequest,
        scope: VrchatScope,
    ) -> VrchatApiFuture<'_> {
        Box::pin(async move {
            vrchat_api::execute_api_command(
                self.web.as_ref(),
                &self.diagnostics,
                &self.sync,
                (&command, detail),
                input,
                scope,
            )
            .await
        })
    }
}
