use crate::imap::{self, ImapConnectionReport, QqImapConnectionInput};

#[tauri::command]
pub fn test_qq_imap_connection(
    input: QqImapConnectionInput,
) -> Result<ImapConnectionReport, String> {
    imap::test_qq_connection(input)
}
