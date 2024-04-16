//! Module which defines backend trait which allows to use different
//! communication channels with control units.

use async_trait::async_trait;
use std::time::Duration;

/// Backend which determines the communication channel with a control unit.
#[async_trait]
pub trait Backend {
    /// Establishes a connection with the control unit.
    async fn connect(&mut self) -> crate::Result<()>;

    /// Drops an already created connection with the control unit.
    async fn disconnect(&mut self) -> crate::Result<()>;

    /// Determines if the backend is currently connected to the control unit.
    async fn is_connected(&self) -> crate::Result<bool>;

    /// Sends a request with the given timeout to the control unit and waits for a response.
    async fn request(&mut self, data: &[u8], timeout: Duration) -> crate::Result<Vec<u8>>;
}
