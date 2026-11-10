pub mod conf;
pub mod initdb;
pub mod paths;
pub struct PgServer;
impl PgServer {
    pub async fn stop(self) -> crate::Result<()> {
        Err(crate::Error::NotImplemented)
    }
}
