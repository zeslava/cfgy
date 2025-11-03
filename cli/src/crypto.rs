use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};
use ring::digest;
use std::fs;

/// Размер ключа шифрования (256 бит для AES-256)
const KEY_SIZE: usize = 32;

/// Длина nonce для AES-GCM (96 бит = 12 байт)
const NONCE_SIZE: usize = 12;

/// Сервис для шифрования секретов
pub struct CryptoService {
    cipher: Aes256Gcm,
}

impl CryptoService {
    /// Создать сервис шифрования на основе мастер-ключа
    pub fn new(master_key: &[u8]) -> Result<Self> {
        // Вывести ключ нужного размера из мастер-ключа используя SHA-256
        let key_bytes = Self::derive_key(master_key);
        
        // Преобразовать в Key для AES-256-GCM
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }

    /// Получить или создать мастер-ключ
    pub fn get_or_create_master_key() -> Result<Vec<u8>> {
        let key_path = Self::master_key_path();
        
        if key_path.exists() {
            let key_hex = fs::read_to_string(&key_path)
                .with_context(|| format!("Не удалось прочитать ключ: {}", key_path.display()))?;
            
            hex::decode(key_hex.trim())
                .with_context(|| "Не удалось декодировать ключ из hex")
        } else {
            // Создать новый ключ
            let key = Self::generate_master_key();
            
            // Сохранить ключ
            if let Some(parent) = key_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Не удалось создать директорию: {}", parent.display()))?;
            }
            
            let key_hex = hex::encode(&key);
            fs::write(&key_path, key_hex)
                .with_context(|| format!("Не удалось сохранить ключ: {}", key_path.display()))?;
            
            // Установить права доступа только для владельца (Unix)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&key_path)?.permissions();
                perms.set_mode(0o600); // rw-------
                fs::set_permissions(&key_path, perms)?;
            }
            
            Ok(key)
        }
    }

    /// Путь к файлу мастер-ключа
    fn master_key_path() -> std::path::PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".cfgy")
            .join("master.key")
    }

    /// Сгенерировать новый мастер-ключ
    fn generate_master_key() -> Vec<u8> {
        let mut key = vec![0u8; KEY_SIZE];
        use ring::rand::{SecureRandom, SystemRandom};
        let rng = SystemRandom::new();
        rng.fill(&mut key).expect("Не удалось сгенерировать ключ");
        key
    }

    /// Вывести ключ из мастер-ключа используя SHA-256
    fn derive_key(master_key: &[u8]) -> Vec<u8> {
        let mut context = digest::Context::new(&digest::SHA256);
        context.update(master_key);
        context.finish().as_ref().to_vec()
    }

    /// Зашифровать данные
    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>> {
        // Сгенерировать случайный nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        // Зашифровать данные
        let ciphertext = self.cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Ошибка шифрования: {}", e))?;
        
        // Объединить nonce и ciphertext: [nonce (12 bytes)][ciphertext]
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(nonce.as_ref());
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    /// Расшифровать данные
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<String> {
        if ciphertext.len() < NONCE_SIZE {
            anyhow::bail!("Данные слишком короткие для расшифровки");
        }
        
        // Извлечь nonce и ciphertext
        let nonce_bytes = &ciphertext[..NONCE_SIZE];
        let ciphertext_only = &ciphertext[NONCE_SIZE..];
        
        let nonce = Nonce::from_slice(nonce_bytes);
        
        // Расшифровать данные
        let plaintext = self.cipher
            .decrypt(&nonce, ciphertext_only)
            .map_err(|e| anyhow::anyhow!("Ошибка расшифровки: {}", e))?;
        
        String::from_utf8(plaintext)
            .map_err(|e| anyhow::anyhow!("Ошибка преобразования в UTF-8: {}", e))
    }
}

