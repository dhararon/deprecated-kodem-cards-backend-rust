use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm, jwk::JwkSet};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use crate::config::FirebaseConfig;
use crate::utils::error::AppError;

const FIREBASE_PUBLIC_KEYS_URL: &str = "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";
const GOOGLE_JWKS_URL: &str = "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";
const KEYS_REFRESH_BUFFER_SECS: u64 = 300; // 5 minutes buffer before expiry

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FirebaseClaims {
    pub sub: String,         // Subject (user ID)
    pub aud: String,         // Audience (project ID)
    pub iss: String,         // Issuer
    pub iat: u64,            // Issued at
    pub exp: u64,            // Expiration time
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Clone)]
pub struct FirebaseAuth {
    project_id: String,
    client: Client,
    keys: Arc<RwLock<CachedKeys>>,
    use_emulator: bool,
    emulator_url: Option<String>,
}

struct CachedKeys {
    jwks: JwkSet,
    expiry: SystemTime,
}

impl FirebaseAuth {
    pub async fn new(config: FirebaseConfig) -> Result<Self, AppError> {
        let client = Client::builder()
            .use_rustls_tls() // Usar rustls en lugar de OpenSSL
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;
        
        // Configurar URL del emulador si está habilitado
        let emulator_url = if config.use_emulator {
            if let (Some(host), Some(port)) = (&config.emulator_host, config.emulator_port) {
                Some(format!("http://{}:{}", host, port))
            } else {
                return Err(AppError::Internal(
                    "Emulador de Firebase habilitado pero falta host o puerto".to_string()
                ));
            }
        } else {
            None
        };
        
        // Si estamos usando el emulador, no necesitamos obtener las claves públicas
        let keys = if config.use_emulator {
            CachedKeys {
                jwks: JwkSet { keys: vec![] },
                expiry: SystemTime::now() + Duration::from_secs(3600),
            }
        } else {
            Self::fetch_keys(&client).await?
        };
        
        tracing::info!(
            "Inicializando Firebase Auth. Proyecto: {}, Emulador: {}",
            config.project_id,
            if config.use_emulator { "Habilitado" } else { "Deshabilitado" }
        );
        
        if config.use_emulator {
            tracing::info!("Usando emulador de Firebase en: {}", emulator_url.as_ref().unwrap());
        }
        
        Ok(Self {
            project_id: config.project_id,
            client,
            keys: Arc::new(RwLock::new(keys)),
            use_emulator: config.use_emulator,
            emulator_url,
        })
    }

    async fn fetch_keys(client: &Client) -> Result<CachedKeys, AppError> {
        // Obtener las claves JWK de Google
        let response = client
            .get(GOOGLE_JWKS_URL)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch Firebase public keys: {}", e)))?;
        
        let cache_control = response
            .headers()
            .get("cache-control")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("max-age=3600");
        
        let max_age = parse_max_age(cache_control).unwrap_or(3600);
        let expiry = SystemTime::now() + Duration::from_secs(max_age - KEYS_REFRESH_BUFFER_SECS);
        
        let jwks: JwkSet = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse Firebase JWK keys: {}", e)))?;
        
        Ok(CachedKeys { jwks, expiry })
    }

    pub async fn verify_token(&self, token: &str) -> Result<FirebaseClaims, AppError> {
        // Si estamos usando el emulador, verificamos el token de manera diferente
        if self.use_emulator {
            return self.verify_emulator_token(token).await;
        }
        
        // Verificación normal para producción
        let header = decode_header(token)
            .map_err(|e| AppError::Authentication(format!("Invalid token header: {}", e)))?;
        
        // Obtener el kid (Key ID) del encabezado
        let kid = header.kid.ok_or_else(|| 
            AppError::Authentication("Token header missing 'kid' claim".to_string())
        )?;
        
        // Obtener la clave pública correspondiente al kid
        let mut keys_guard = self.keys.write().await;
        
        // Verificar si las claves necesitan actualizarse
        if SystemTime::now() >= keys_guard.expiry {
            *keys_guard = Self::fetch_keys(&self.client).await?;
        }
        
        // Buscar la clave JWK correspondiente al kid
        let jwk = keys_guard.jwks.find(&kid)
            .ok_or_else(|| AppError::Authentication(format!("No matching key found for kid: {}", kid)))?;
        
        // Convertir JWK a DecodingKey
        let decoding_key = DecodingKey::from_jwk(jwk)
            .map_err(|e| AppError::Authentication(format!("Failed to create decoding key: {}", e)))?;
        
        // Liberar el lock
        drop(keys_guard);
        
        // Configurar la validación
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.project_id]);
        validation.set_issuer(&[&format!("https://securetoken.google.com/{}", self.project_id)]);
        
        // Verificar el token
        let token_data = decode::<FirebaseClaims>(token, &decoding_key, &validation)
            .map_err(|e| AppError::Authentication(format!("Invalid token: {}", e)))?;
        
        // Verificar que el token no haya expirado
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if token_data.claims.exp < now {
            return Err(AppError::Authentication("Token has expired".to_string()));
        }
        
        Ok(token_data.claims)
    }
    
    async fn verify_emulator_token(&self, token: &str) -> Result<FirebaseClaims, AppError> {
        // En el emulador, simplemente decodificamos el token sin verificar la firma
        // Esto es seguro solo para desarrollo local
        
        // Dividir el token en sus partes
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(AppError::Authentication("Token de formato inválido".to_string()));
        }
        
        // Decodificar la parte del payload
        let payload_base64 = parts[1];
        let payload_json = base64_decode(payload_base64)
            .map_err(|_| AppError::Authentication("No se pudo decodificar el payload del token".to_string()))?;
        
        let claims: FirebaseClaims = serde_json::from_slice(&payload_json)
            .map_err(|e| AppError::Authentication(format!("Payload del token no es válido: {}", e)))?;
        
        tracing::info!("Verificación de token en emulador: {:?}", claims);
        
        Ok(claims)
    }
}

impl Default for FirebaseAuth {
    fn default() -> Self {
        let client = Client::builder()
            .use_rustls_tls()
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            project_id: "kodemcards".to_string(),
            client,
            keys: Arc::new(RwLock::new(CachedKeys {
                jwks: JwkSet { keys: vec![] },
                expiry: SystemTime::now() + Duration::from_secs(3600),
            })),
            use_emulator: true,
            emulator_url: Some("http://localhost:9099".to_string()),
        }
    }
}

fn parse_max_age(cache_control: &str) -> Option<u64> {
    cache_control
        .split(',')
        .find(|directive| directive.trim().starts_with("max-age="))
        .and_then(|directive| {
            directive
                .trim()
                .strip_prefix("max-age=")
                .and_then(|age| age.parse::<u64>().ok())
        })
}

// Función para decodificar base64url a bytes
fn base64_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    // Ajustar el padding si es necesario
    let input = match input.len() % 4 {
        0 => input.to_string(),
        2 => format!("{}==", input),
        3 => format!("{}=", input),
        _ => return Err("Longitud de base64 inválida"),
    };
    
    // Reemplazar caracteres especiales de base64url a base64 estándar
    let input = input.replace('-', "+").replace('_', "/");
    
    // Decodificar
    base64::decode(&input).map_err(|_| "Error al decodificar base64")
}
