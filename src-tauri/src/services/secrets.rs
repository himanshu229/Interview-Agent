use keyring::Entry;

use crate::error::AppResult;

const SERVICE: &str = "com.aidesktop.assistant";
const ACCOUNT: &str = "openai-api-key";

/// Wraps the OS keychain for secure storage of the OpenAI API key.
/// The key is never written to disk or config files in plaintext.
pub struct SecretStore;

impl SecretStore {
    fn entry() -> keyring::Result<Entry> {
        Entry::new(SERVICE, ACCOUNT)
    }

    pub fn set_api_key(key: &str) -> AppResult<()> {
        Self::entry()?.set_password(key)?;
        Ok(())
    }

    pub fn get_api_key() -> AppResult<Option<String>> {
        match Self::entry()?.get_password() {
            Ok(k) => Ok(Some(k)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn clear_api_key() -> AppResult<()> {
        match Self::entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn has_api_key() -> bool {
        matches!(Self::get_api_key(), Ok(Some(_)))
    }
}
