use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub server_port: u16,
    pub environment: String,
    pub firebase: FirebaseConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FirebaseConfig {
    pub project_id: String,
    pub api_key: String,
    pub auth_domain: String,
    pub use_emulator: bool,
    pub emulator_host: Option<String>,
    pub emulator_port: Option<u16>,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        // Determinar si se debe usar el emulador de Firebase
        let use_emulator = env::var("USE_FIREBASE_EMULATOR")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        
        // Obtener la configuración del emulador solo si se va a usar
        let (emulator_host, emulator_port) = if use_emulator {
            (
                Some(env::var("FIREBASE_EMULATOR_HOST").unwrap_or_else(|_| "localhost".to_string())),
                Some(
                    env::var("FIREBASE_AUTH_EMULATOR_PORT")
                        .unwrap_or_else(|_| "9099".to_string())
                        .parse::<u16>()
                        .unwrap_or(9099)
                ),
            )
        } else {
            (None, None)
        };

        Ok(Config {
            database_url: env::var("DATABASE_URL")?,
            redis_url: env::var("REDIS_URL")?,
            jwt_secret: env::var("JWT_SECRET")?,
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            firebase: FirebaseConfig {
                project_id: env::var("FIREBASE_PROJECT_ID")?,
                api_key: env::var("FIREBASE_API_KEY")?,
                auth_domain: env::var("FIREBASE_AUTH_DOMAIN")?,
                use_emulator,
                emulator_host,
                emulator_port,
            },
        })
    }
}

pub mod firebase;
