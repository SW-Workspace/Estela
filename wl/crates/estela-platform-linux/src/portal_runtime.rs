use std::sync::OnceLock;

use tokio::{runtime::Runtime, sync::OnceCell};

static RUNTIME: OnceLock<std::result::Result<Runtime, String>> = OnceLock::new();
static REGISTRATION: OnceCell<()> = OnceCell::const_new();

pub fn runtime() -> std::result::Result<&'static Runtime, String> {
    RUNTIME
        .get_or_init(|| Runtime::new().map_err(|error| error.to_string()))
        .as_ref()
        .map_err(Clone::clone)
}

pub async fn register_application() {
    REGISTRATION
        .get_or_init(|| async {
            if let Ok(id) = "app.estela.Estela".try_into() {
                // Older portals can identify installed applications without Registry.
                let _ = ashpd::register_host_app(id).await;
            }
        })
        .await;
}
