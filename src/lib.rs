pub mod device;
pub mod hub;
pub mod scene;

pub use device::{Device, DeviceData, DeviceType};
pub use scene::Scene;

use serde::Deserialize;

pub(crate) fn deserialize_datetime<'de, D>(
    deserializer: D,
) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let date_str = String::deserialize(deserializer)?;
    match date_str.parse() {
        Ok(system_time) => Ok(system_time),
        Err(_) => Err(serde::de::Error::custom("Invalid date format")),
    }
}

pub(crate) fn deserialize_datetime_optional<'de, D>(
    deserializer: D,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match deserialize_datetime(deserializer) {
        Ok(system_time) => Ok(Some(system_time)),
        Err(_) => Ok(None),
    }
}

pub mod danger {
    pub struct NoCertificateVerification;

    impl rustls::client::ServerCertVerifier for NoCertificateVerification {
        fn verify_server_cert(
            &self,
            _end_entity: &rustls::Certificate,
            _intermediates: &[rustls::Certificate],
            _server_name: &rustls::client::ServerName,
            _scts: &mut dyn Iterator<Item = &[u8]>,
            _ocsp_response: &[u8],
            _now: std::time::SystemTime,
        ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
            Ok(rustls::client::ServerCertVerified::assertion())
        }
    }

    pub fn tls_no_verify() -> rustls::ClientConfig {
        let mut tls = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(rustls::RootCertStore::empty())
            .with_no_client_auth();

        tls.dangerous()
            .set_certificate_verifier(std::sync::Arc::new(NoCertificateVerification));

        tls
    }
}
