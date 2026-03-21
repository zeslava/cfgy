use anyhow::Result;
use async_trait::async_trait;

/// Трейт для криптографических операций
#[async_trait]
pub trait CryptoService: Send + Sync {
    /// Зашифровать данные
    async fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>>;

    /// Расшифровать данные
    async fn decrypt(&self, ciphertext: &[u8]) -> Result<String>;
}
