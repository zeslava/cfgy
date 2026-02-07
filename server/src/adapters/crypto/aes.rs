use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use anyhow::Result;
use async_trait::async_trait;
use ring::digest;

use crate::ports::CryptoService;

/// Размер ключа шифрования (256 бит для AES-256)
#[allow(dead_code)]
const KEY_SIZE: usize = 32;

/// Длина nonce для AES-GCM (96 бит = 12 байт)
const NONCE_SIZE: usize = 12;

use std::sync::Arc;

/// Реализация криптографического сервиса на основе AES-GCM
#[derive(Clone)]
pub struct AesCryptoService {
    cipher: Arc<Aes256Gcm>,
}

impl AesCryptoService {
    /// Создать сервис шифрования на основе мастер-ключа
    pub fn new(master_key: &[u8]) -> Result<Self> {
        // Вывести ключ нужного размера из мастер-ключа используя SHA-256
        let key_bytes = Self::derive_key(master_key);

        // Преобразовать в Key для AES-256-GCM
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        Ok(Self {
            cipher: Arc::new(cipher),
        })
    }

    /// Вывести ключ из мастер-ключа используя SHA-256
    fn derive_key(master_key: &[u8]) -> Vec<u8> {
        let mut context = digest::Context::new(&digest::SHA256);
        context.update(master_key);
        context.finish().as_ref().to_vec()
    }
}

#[async_trait]
impl CryptoService for AesCryptoService {
    /// Зашифровать данные
    async fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>> {
        // Сгенерировать случайный nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // Зашифровать данные
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Ошибка шифрования: {}", e))?;

        // Объединить nonce и ciphertext: [nonce (12 bytes)][ciphertext]
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(nonce.as_ref());
        result.extend_from_slice(&ciphertext);

        tracing::debug!(
            "Зашифровано данных: размер plaintext={}, размер результата={}, nonce={:?}",
            plaintext.len(),
            result.len(),
            nonce.as_ref() as &[u8]
        );

        Ok(result)
    }

    /// Расшифровать данные
    async fn decrypt(&self, ciphertext: &[u8]) -> Result<String> {
        if ciphertext.len() < NONCE_SIZE {
            tracing::error!(
                "Данные слишком короткие для расшифровки: {} байт (требуется минимум {} байт)",
                ciphertext.len(),
                NONCE_SIZE
            );
            anyhow::bail!(
                "Данные слишком короткие для расшифровки: {} байт (требуется минимум {} байт)",
                ciphertext.len(),
                NONCE_SIZE
            );
        }

        // Извлечь nonce и ciphertext
        let nonce_bytes = &ciphertext[..NONCE_SIZE];
        let ciphertext_only = &ciphertext[NONCE_SIZE..];

        let nonce = Nonce::from_slice(nonce_bytes);

        // Расшифровать данные
        let plaintext = self
            .cipher
            .decrypt(&nonce, ciphertext_only)
            .map_err(|e| {
                tracing::error!("Ошибка расшифровки AES-GCM: {} (размер данных: {} байт, размер nonce: {} байт, размер ciphertext: {} байт)", 
                    e, ciphertext.len(), NONCE_SIZE, ciphertext_only.len());
                anyhow::anyhow!("Ошибка расшифровки: {} (возможно, изменился master key или данные повреждены)", e)
            })?;

        String::from_utf8(plaintext).map_err(|e| {
            tracing::error!("Ошибка преобразования расшифрованных данных в UTF-8: {}", e);
            anyhow::anyhow!("Ошибка преобразования в UTF-8: {}", e)
        })
    }
}
