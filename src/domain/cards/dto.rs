use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize, Deserializer};
use uuid::Uuid;
use anyhow::{Result, anyhow};

use super::model::CardSet;

pub trait Validable {
    fn validate(&self) -> Result<()>;
}

// Función personalizada para deserializar fechas en múltiples formatos
fn flexible_date_format<'de, D>(deserializer: D) -> std::result::Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let date_str = String::deserialize(deserializer)?;
    
    // Primero, intentamos analizar como un DateTime completo
    if let Ok(date_time) = DateTime::parse_from_rfc3339(&date_str) {
        return Ok(date_time.with_timezone(&Utc));
    }
    
    // Si falla, intentamos analizar como fecha simple YYYY-MM-DD
    if let Ok(naive_date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
        // Convertimos a DateTime con tiempo a medianoche en UTC
        let naive_datetime = naive_date.and_hms_opt(0, 0, 0).unwrap();
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc));
    }
    
    // Si todo falla, reportamos el error
    Err(serde::de::Error::custom(format!(
        "El formato de fecha '{}' es inválido. Usa el formato ISO 8601 (por ejemplo, '2025-01-01' o '2025-01-01T00:00:00Z')",
        date_str
    )))
}

// Versión opcional para deserializar fechas que pueden ser nulas
fn flexible_date_format_optional<'de, D>(deserializer: D) -> std::result::Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    // Primero intentamos deserializar como un Option<String>
    let opt = Option::<String>::deserialize(deserializer)?;
    
    match opt {
        // Si no hay fecha, retornamos None
        None => Ok(None),
        // Si hay una fecha, la parseamos
        Some(date_str) => {
            // Primero, intentamos analizar como un DateTime completo
            if let Ok(date_time) = DateTime::parse_from_rfc3339(&date_str) {
                return Ok(Some(date_time.with_timezone(&Utc)));
            }
            
            // Si falla, intentamos analizar como fecha simple YYYY-MM-DD
            if let Ok(naive_date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                // Convertimos a DateTime con tiempo a medianoche en UTC
                let naive_datetime = naive_date.and_hms_opt(0, 0, 0).unwrap();
                return Ok(Some(DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc)));
            }
            
            // Si todo falla, reportamos el error
            Err(serde::de::Error::custom(format!(
                "El formato de fecha '{}' es inválido. Usa el formato ISO 8601 (por ejemplo, '2025-01-01' o '2025-01-01T00:00:00Z')",
                date_str
            )))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCardSetDto {
    pub name: String,
    pub code: String,
    #[serde(deserialize_with = "flexible_date_format")]
    pub release_date: DateTime<Utc>,
    pub icon_url: Option<String>,
    pub total_cards: i32,
}

impl CreateCardSetDto {
    pub fn to_model(&self) -> CardSet {
        CardSet::new(
            self.name.clone(),
            self.code.clone(),
            self.release_date,
            self.icon_url.clone(),
            self.total_cards,
        )
    }
}

impl Validable for CreateCardSetDto {
    fn validate(&self) -> Result<()> {
        // Validar nombre (no vacío y longitud mínima)
        if self.name.trim().is_empty() {
            return Err(anyhow!("El nombre no puede estar vacío"));
        }
        
        if self.name.len() < 3 {
            return Err(anyhow!("El nombre debe tener al menos 3 caracteres"));
        }
        
        if self.name.len() > 100 {
            return Err(anyhow!("El nombre no puede exceder los 100 caracteres"));
        }
        
        // Validar código (formato y longitud)
        if self.code.trim().is_empty() {
            return Err(anyhow!("El código no puede estar vacío"));
        }
        
        if self.code.len() < 2 || self.code.len() > 10 {
            return Err(anyhow!("El código debe tener entre 2 y 10 caracteres"));
        }
        
        // Verifica que el código esté en mayúsculas
        if self.code != self.code.to_uppercase() {
            return Err(anyhow!("El código debe estar en mayúsculas"));
        }
        
        // Validar total_cards (mayor que cero)
        if self.total_cards <= 0 {
            return Err(anyhow!("El número total de cartas debe ser mayor que cero"));
        }
        
        // Validar que la fecha de lanzamiento no sea futura
        let now = Utc::now();
        if self.release_date > now && (self.release_date - now).num_days() > 365 {
            return Err(anyhow!("La fecha de lanzamiento no puede ser más de un año en el futuro"));
        }
        
        // Validar URL del ícono si está presente
        if let Some(url) = &self.icon_url {
            if url.trim().is_empty() {
                return Err(anyhow!("La URL del ícono no puede estar vacía"));
            }
            
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(anyhow!("La URL del ícono debe comenzar con http:// o https://"));
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCardSetDto {
    pub name: String,
    pub code: String,
    #[serde(deserialize_with = "flexible_date_format")]
    pub release_date: DateTime<Utc>,
    pub icon_url: Option<String>,
    pub total_cards: i32,
}

impl UpdateCardSetDto {
    pub fn to_model(&self, id: Uuid, created_at: DateTime<Utc>) -> CardSet {
        CardSet {
            id,
            name: self.name.clone(),
            code: self.code.clone(),
            release_date: self.release_date,
            icon_url: self.icon_url.clone(),
            total_cards: self.total_cards,
            created_at,
            updated_at: Utc::now(),
        }
    }
}

impl Validable for UpdateCardSetDto {
    fn validate(&self) -> Result<()> {
        // Validar nombre (no vacío y longitud mínima)
        if self.name.trim().is_empty() {
            return Err(anyhow!("El nombre no puede estar vacío"));
        }
        
        if self.name.len() < 3 {
            return Err(anyhow!("El nombre debe tener al menos 3 caracteres"));
        }
        
        if self.name.len() > 100 {
            return Err(anyhow!("El nombre no puede exceder los 100 caracteres"));
        }
        
        // Validar código (formato y longitud)
        if self.code.trim().is_empty() {
            return Err(anyhow!("El código no puede estar vacío"));
        }
        
        if self.code.len() < 2 || self.code.len() > 10 {
            return Err(anyhow!("El código debe tener entre 2 y 10 caracteres"));
        }
        
        // Verifica que el código esté en mayúsculas
        if self.code != self.code.to_uppercase() {
            return Err(anyhow!("El código debe estar en mayúsculas"));
        }
        
        // Validar total_cards (mayor que cero)
        if self.total_cards <= 0 {
            return Err(anyhow!("El número total de cartas debe ser mayor que cero"));
        }
        
        // Validar que la fecha de lanzamiento no sea futura
        let now = Utc::now();
        if self.release_date > now && (self.release_date - now).num_days() > 365 {
            return Err(anyhow!("La fecha de lanzamiento no puede ser más de un año en el futuro"));
        }
        
        // Validar URL del ícono si está presente
        if let Some(url) = &self.icon_url {
            if url.trim().is_empty() {
                return Err(anyhow!("La URL del ícono no puede estar vacía"));
            }
            
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(anyhow!("La URL del ícono debe comenzar con http:// o https://"));
            }
        }
        
        Ok(())
    }
}

// DTO para actualizaciones parciales (PATCH)
#[derive(Debug, Serialize, Deserialize)]
pub struct PatchCardSetDto {
    pub name: Option<String>,
    pub code: Option<String>,
    #[serde(default, deserialize_with = "flexible_date_format_optional")]
    pub release_date: Option<DateTime<Utc>>,
    pub icon_url: Option<Option<String>>, // Option<Option<>> para permitir eliminar el valor (null) o no incluirlo
    pub total_cards: Option<i32>,
}

impl PatchCardSetDto {
    pub fn apply_to_model(&self, mut card_set: CardSet) -> CardSet {
        // Sólo actualizamos los campos que están presentes en el DTO
        if let Some(name) = &self.name {
            card_set.name = name.clone();
        }
        
        if let Some(code) = &self.code {
            card_set.code = code.clone();
        }
        
        if let Some(release_date) = self.release_date {
            card_set.release_date = release_date;
        }
        
        // Manejo especial para icon_url, que es Option<Option<String>>
        // Esto permite distinguir entre "no actualizar" y "establecer en null"
        if let Some(icon_url) = &self.icon_url {
            card_set.icon_url = icon_url.clone();
        }
        
        if let Some(total_cards) = self.total_cards {
            card_set.total_cards = total_cards;
        }
        
        // Siempre actualizamos la fecha de actualización
        card_set.updated_at = Utc::now();
        
        card_set
    }
}

impl Validable for PatchCardSetDto {
    fn validate(&self) -> Result<()> {
        // Solo validamos los campos que están presentes
        if let Some(name) = &self.name {
            if name.trim().is_empty() {
                return Err(anyhow!("El nombre no puede estar vacío"));
            }
            
            if name.len() < 3 {
                return Err(anyhow!("El nombre debe tener al menos 3 caracteres"));
            }
            
            if name.len() > 100 {
                return Err(anyhow!("El nombre no puede exceder los 100 caracteres"));
            }
        }
        
        if let Some(code) = &self.code {
            if code.trim().is_empty() {
                return Err(anyhow!("El código no puede estar vacío"));
            }
            
            if code.len() < 2 || code.len() > 10 {
                return Err(anyhow!("El código debe tener entre 2 y 10 caracteres"));
            }
            
            // Verifica que el código esté en mayúsculas
            if code != &code.to_uppercase() {
                return Err(anyhow!("El código debe estar en mayúsculas"));
            }
        }
        
        if let Some(total_cards) = self.total_cards {
            if total_cards <= 0 {
                return Err(anyhow!("El número total de cartas debe ser mayor que cero"));
            }
        }
        
        if let Some(release_date) = self.release_date {
            // Validar que la fecha de lanzamiento no sea futura
            let now = Utc::now();
            if release_date > now && (release_date - now).num_days() > 365 {
                return Err(anyhow!("La fecha de lanzamiento no puede ser más de un año en el futuro"));
            }
        }
        
        // Validar URL del ícono si está presente y no es None
        if let Some(Some(url)) = &self.icon_url {
            if url.trim().is_empty() {
                return Err(anyhow!("La URL del ícono no puede estar vacía"));
            }
            
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(anyhow!("La URL del ícono debe comenzar con http:// o https://"));
            }
        }
        
        Ok(())
    }
} 