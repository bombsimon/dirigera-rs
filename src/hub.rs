use serde::Deserialize;
use std::collections::HashMap;

#[cfg(feature = "config")]
use std::io::Read;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Hub {
    ip_address: std::net::Ipv4Addr,
    token: String,
}

const DIRIGERA_PORT: u16 = 8443;
const DIRIGERA_API_VERSION: &str = "v1";

/// The requests to the Dirigera hub is a list of maps with attributes to set. F.ex. to turn on a
/// light bulb you would pass
///
///     [{"attributes": {"isOn": true}}]
type DirigeraBody<'a, T> = http::Request<Vec<HashMap<&'a str, HashMap<&'a str, T>>>>;

#[cfg(feature = "config")]
impl Default for Hub {
    fn default() -> Self {
        let mut toml_content = String::new();
        std::fs::File::open("config.toml")
            .expect("Failed to open config.toml")
            .read_to_string(&mut toml_content)
            .expect("Failed to read config.toml");

        toml::from_str(&toml_content).expect("Failed to parse TOML")
    }
}

impl Hub {
    pub fn new(ip_address: std::net::Ipv4Addr, token: String) -> Self {
        Hub { ip_address, token }
    }

    fn build_uri<T>(
        &self,
        method: http::Method,
        path: &str,
        body: T,
    ) -> anyhow::Result<http::Request<T>>
    where
        T: serde::Serialize,
    {
        let url = format!(
            "https://{}:{}/{}{}",
            self.ip_address, DIRIGERA_PORT, DIRIGERA_API_VERSION, path
        );

        http::Request::builder()
            .uri(url)
            .method(method)
            .header("User-Agent", "dirigera-rs/0.1.0")
            .header("Authorization", format!("Bearer {}", self.token))
            .body(body)
            .map_err(|err| anyhow::anyhow!(err))
    }

    pub fn devices_request(&self) -> anyhow::Result<http::Request<()>> {
        self.build_uri(http::Method::GET, "/devices", ())
    }

    pub fn devices_parse_response<T>(
        &self,
        response: http::Response<T>,
    ) -> anyhow::Result<Vec<crate::Device>>
    where
        T: AsRef<[u8]>,
    {
        serde_json::from_slice(response.body().as_ref()).map_err(|err| anyhow::anyhow!(err))
    }

    pub fn device_request(&self, id: &str) -> anyhow::Result<http::Request<()>> {
        self.build_uri(http::Method::GET, format!("/devices/{}", id).as_str(), ())
    }

    pub fn device_parse_response<T>(
        &self,
        response: http::Response<T>,
    ) -> anyhow::Result<crate::Device>
    where
        T: AsRef<[u8]>,
    {
        serde_json::from_slice(response.body().as_ref()).map_err(|err| anyhow::anyhow!(err))
    }

    pub fn scenes_request(&self) -> anyhow::Result<http::Request<()>> {
        self.build_uri(http::Method::GET, "/scenes", ())
    }

    pub fn scenes_parse_response<T>(
        &self,
        response: http::Response<T>,
    ) -> anyhow::Result<Vec<crate::Scene>>
    where
        T: AsRef<[u8]>,
    {
        serde_json::from_slice(response.body().as_ref()).map_err(|err| anyhow::anyhow!(err))
    }

    pub fn device_reload_request(
        &self,
        device: &crate::device::Device,
    ) -> anyhow::Result<http::Request<()>> {
        self.build_uri(
            http::Method::GET,
            format!("/devices/{}", device.into_inner().id).as_str(),
            (),
        )
    }

    pub fn device_reload_parse_response<T>(
        &self,
        device: &mut crate::device::Device,
        response: http::Response<T>,
    ) -> anyhow::Result<()>
    where
        T: AsRef<[u8]>,
    {
        *device =
            serde_json::from_slice(response.body().as_ref()).map_err(|err| anyhow::anyhow!(err))?;

        Ok(())
    }

    pub fn device_toggle_request(
        &self,
        device: &crate::device::Device,
    ) -> anyhow::Result<DirigeraBody<bool>> {
        if !device
            .into_inner()
            .capabilities
            .can_receive
            .contains(&crate::device::Capability::IsOn)
        {
            anyhow::bail!("device cannot be toggled");
        }

        let inner = device.into_inner();

        let mut attributes = HashMap::new();
        attributes.insert("isOn", !inner.attributes.is_on);

        let mut body = HashMap::new();
        body.insert("attributes", attributes);

        self.build_uri(
            http::Method::PATCH,
            format!("/devices/{}", inner.id).as_str(),
            vec![body],
        )
    }

    pub fn device_set_hue_saturation(
        &self,
        device: &crate::device::Device,
        hue: f64,
        saturation: f64,
    ) -> anyhow::Result<DirigeraBody<f64>> {
        if !device
            .into_inner()
            .capabilities
            .can_receive
            .contains(&crate::device::Capability::ColorHue)
        {
            anyhow::bail!("device cannot be toggled");
        }

        let inner = device.into_inner();

        let mut attributes = HashMap::new();
        attributes.insert("colorHue", hue);
        attributes.insert("colorSaturation", saturation);

        let mut body = HashMap::new();
        body.insert("attributes", attributes);

        self.build_uri(
            http::Method::PATCH,
            format!("/devices/{}", inner.id).as_str(),
            vec![body],
        )
    }
}
