use super::{MailProvider, ProviderSyncPayload};
use crate::imap::{self, QqInboxSyncInput};
use crate::models::ProviderKind;

pub(crate) struct QqImapProvider;

impl MailProvider for QqImapProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Qq
    }

    fn sync_inbox(
        &self,
        address: &str,
        secret: &str,
        limit: Option<u32>,
    ) -> Result<ProviderSyncPayload, String> {
        let payload = imap::sync_qq_inbox(QqInboxSyncInput {
            email: address.to_owned(),
            authorization_code: secret.to_owned(),
            limit,
        })?;

        Ok(ProviderSyncPayload {
            folder: payload.report.folder,
            messages: payload.messages,
            total_messages: payload.report.exists,
        })
    }
}
