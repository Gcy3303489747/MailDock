mod qq;

use crate::models::{MailMessage, ProviderKind};

pub(crate) use qq::QqImapProvider;

pub(crate) struct ProviderSyncPayload {
    pub folder: String,
    pub messages: Vec<MailMessage>,
    pub total_messages: u32,
}

pub(crate) trait MailProvider {
    fn kind(&self) -> ProviderKind;
    fn sync_inbox(
        &self,
        address: &str,
        secret: &str,
        limit: Option<u32>,
    ) -> Result<ProviderSyncPayload, String>;
}
