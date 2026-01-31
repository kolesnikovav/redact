use serde::{Deserialize, Serialize};
use std::fmt;

/// Entity types supported by the PII detection engine
/// Compatible with Microsoft Presidio entity types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityType {
    // Personal identifiers (NER-based)
    Person,
    Location,
    Organization,
    DateTime,

    // Contact information
    EmailAddress,
    PhoneNumber,
    IpAddress,
    Url,
    DomainName,

    // Financial
    CreditCard,
    Iban,
    IbanCode,
    UsBankNumber,

    // US-specific identifiers
    UsSsn,
    UsDriverLicense,
    UsPassport,
    UsZipCode,

    // UK-specific identifiers
    UkNhs,
    UkNino,
    UkPostcode,
    UkDriverLicense,
    UkPassportNumber,
    UkPhoneNumber,
    UkMobileNumber,
    UkSortCode,
    UkCompanyNumber,

    // Healthcare
    MedicalLicense,
    MedicalRecordNumber,

    // Generic identifiers
    PassportNumber, // Generic, non-country specific
    Age,
    Isbn,
    PoBox,

    // Crypto
    CryptoWallet,
    BtcAddress,
    EthAddress,

    // Technical
    Guid,
    MacAddress,
    Md5Hash,
    Sha1Hash,
    Sha256Hash,

    // Generic
    Custom(String),
}

impl EntityType {
    /// Get the string representation for the entity type
    pub fn as_str(&self) -> &str {
        match self {
            EntityType::Person => "PERSON",
            EntityType::Location => "LOCATION",
            EntityType::Organization => "ORGANIZATION",
            EntityType::DateTime => "DATE_TIME",
            EntityType::EmailAddress => "EMAIL_ADDRESS",
            EntityType::PhoneNumber => "PHONE_NUMBER",
            EntityType::IpAddress => "IP_ADDRESS",
            EntityType::Url => "URL",
            EntityType::DomainName => "DOMAIN_NAME",
            EntityType::CreditCard => "CREDIT_CARD",
            EntityType::Iban => "IBAN",
            EntityType::IbanCode => "IBAN_CODE",
            EntityType::UsBankNumber => "US_BANK_NUMBER",
            EntityType::UsSsn => "US_SSN",
            EntityType::UsDriverLicense => "US_DRIVER_LICENSE",
            EntityType::UsPassport => "US_PASSPORT",
            EntityType::UsZipCode => "US_ZIP_CODE",
            EntityType::UkNhs => "UK_NHS",
            EntityType::UkNino => "UK_NINO",
            EntityType::UkPostcode => "UK_POSTCODE",
            EntityType::UkDriverLicense => "UK_DRIVER_LICENSE",
            EntityType::UkPassportNumber => "UK_PASSPORT_NUMBER",
            EntityType::UkPhoneNumber => "UK_PHONE_NUMBER",
            EntityType::UkMobileNumber => "UK_MOBILE_NUMBER",
            EntityType::UkSortCode => "UK_SORT_CODE",
            EntityType::UkCompanyNumber => "UK_COMPANY_NUMBER",
            EntityType::MedicalLicense => "MEDICAL_LICENSE",
            EntityType::MedicalRecordNumber => "MEDICAL_RECORD_NUMBER",
            EntityType::PassportNumber => "PASSPORT_NUMBER",
            EntityType::Age => "AGE",
            EntityType::Isbn => "ISBN",
            EntityType::PoBox => "PO_BOX",
            EntityType::CryptoWallet => "CRYPTO_WALLET",
            EntityType::BtcAddress => "BTC_ADDRESS",
            EntityType::EthAddress => "ETH_ADDRESS",
            EntityType::Guid => "GUID",
            EntityType::MacAddress => "MAC_ADDRESS",
            EntityType::Md5Hash => "MD5_HASH",
            EntityType::Sha1Hash => "SHA1_HASH",
            EntityType::Sha256Hash => "SHA256_HASH",
            EntityType::Custom(name) => name,
        }
    }

    /// Get the default replacement text for this entity type
    pub fn default_replacement(&self) -> String {
        format!("[{}]", self.as_str())
    }

    /// Check if this is a high-sensitivity entity requiring elevated protection
    pub fn is_high_sensitivity(&self) -> bool {
        matches!(
            self,
            EntityType::UsSsn
                | EntityType::CreditCard
                | EntityType::UsBankNumber
                | EntityType::UsPassport
                | EntityType::UkNhs
                | EntityType::UkNino
                | EntityType::MedicalLicense
        )
    }
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<String> for EntityType {
    fn from(s: String) -> Self {
        match s.to_uppercase().as_str() {
            "PERSON" => EntityType::Person,
            "LOCATION" => EntityType::Location,
            "ORGANIZATION" => EntityType::Organization,
            "DATE_TIME" => EntityType::DateTime,
            "EMAIL_ADDRESS" => EntityType::EmailAddress,
            "PHONE_NUMBER" => EntityType::PhoneNumber,
            "IP_ADDRESS" => EntityType::IpAddress,
            "URL" => EntityType::Url,
            "DOMAIN_NAME" => EntityType::DomainName,
            "CREDIT_CARD" => EntityType::CreditCard,
            "IBAN" | "IBAN_CODE" => EntityType::IbanCode,
            "US_SSN" => EntityType::UsSsn,
            "US_DRIVER_LICENSE" => EntityType::UsDriverLicense,
            "US_PASSPORT" => EntityType::UsPassport,
            "US_BANK_NUMBER" => EntityType::UsBankNumber,
            "UK_NHS" => EntityType::UkNhs,
            "UK_NINO" => EntityType::UkNino,
            "UK_POSTCODE" => EntityType::UkPostcode,
            "UK_DRIVER_LICENSE" => EntityType::UkDriverLicense,
            "UK_PASSPORT_NUMBER" => EntityType::UkPassportNumber,
            "UK_PHONE_NUMBER" => EntityType::UkPhoneNumber,
            "UK_MOBILE_NUMBER" => EntityType::UkMobileNumber,
            "UK_SORT_CODE" => EntityType::UkSortCode,
            "UK_COMPANY_NUMBER" => EntityType::UkCompanyNumber,
            "MEDICAL_LICENSE" => EntityType::MedicalLicense,
            "CRYPTO_WALLET" => EntityType::CryptoWallet,
            "BTC_ADDRESS" => EntityType::BtcAddress,
            "ETH_ADDRESS" => EntityType::EthAddress,
            "GUID" => EntityType::Guid,
            "MAC_ADDRESS" => EntityType::MacAddress,
            "MD5_HASH" => EntityType::Md5Hash,
            "SHA1_HASH" => EntityType::Sha1Hash,
            "SHA256_HASH" => EntityType::Sha256Hash,
            _ => EntityType::Custom(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_as_str() {
        assert_eq!(EntityType::Person.as_str(), "PERSON");
        assert_eq!(EntityType::UsSsn.as_str(), "US_SSN");
        assert_eq!(EntityType::EmailAddress.as_str(), "EMAIL_ADDRESS");
    }

    #[test]
    fn test_entity_type_default_replacement() {
        assert_eq!(EntityType::Person.default_replacement(), "[PERSON]");
        assert_eq!(EntityType::UsSsn.default_replacement(), "[US_SSN]");
    }

    #[test]
    fn test_high_sensitivity() {
        assert!(EntityType::UsSsn.is_high_sensitivity());
        assert!(EntityType::CreditCard.is_high_sensitivity());
        assert!(!EntityType::EmailAddress.is_high_sensitivity());
    }

    #[test]
    fn test_from_string() {
        assert_eq!(EntityType::from("PERSON".to_string()), EntityType::Person);
        assert_eq!(EntityType::from("person".to_string()), EntityType::Person);
        assert_eq!(EntityType::from("US_SSN".to_string()), EntityType::UsSsn);
    }
}
