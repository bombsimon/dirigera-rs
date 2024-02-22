//! The IKEA hub is what you're communicating with and what's running the API to manage your
//! devices. Because of that the [`Hub`] is what's exposing all methods from the API. The API is a
//! RESTful HTTPS API with a self signed certificate so you need a [`hyper`] client that doesn't do
//! TLS verification. You also need a bearer token which is obtain via OAuth 2. Configuration for
//! TLS and tool to get a token is both available under the [`danger`](crate::danger) module and the
//! `config` feature flag respectively.
use hyper::service::Service;
use serde::Deserialize;

use std::collections::HashMap;
#[cfg(feature = "config")]
use std::io::Read;

const DIRIGERA_PORT: u16 = 8443;
const DIRIGERA_API_VERSION: &str = "v1";

/// A [`Hub`] consists of a [`hyper`] client, the hub's IP address and a token to communicate with
/// it.
#[derive(Debug)]
pub struct Hub {
    client: hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    ip_address: std::net::Ipv4Addr,
    token: String,
}

/// If you want to read the configuration from a `toml` file, the [`Config`] is used to deserialize
/// the file contents. It's only available behind the `config` feature flag.
#[cfg(feature = "config")]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    ip_address: std::net::Ipv4Addr,
    token: String,
}

/// The default implementation for [`Hub`] can be used to read the IP address and token from a
/// `toml` file. Such `toml` file will be created by running the `generate-token` binary. It will
/// also use the [`danger`](crate::danger) module to setup [`rustls`] with no certification
/// verification.
#[cfg(feature = "config")]
impl Default for Hub {
    fn default() -> Self {
        let mut toml_content = String::new();
        std::fs::File::open("config.toml")
            .expect("Failed to open config.toml")
            .read_to_string(&mut toml_content)
            .expect("Failed to read config.toml");

        let config: Config = toml::from_str(&toml_content).expect("Failed to parse TOML");

        let tls = crate::danger::tls_no_verify();
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_tls_config(tls)
            .https_only()
            .enable_http1()
            .build();

        let client = hyper::Client::builder().build::<_, hyper::Body>(https);

        Self::new(client, config.ip_address, config.token)
    }
}

impl Hub {
    /// Create a new instance of the [`Hub`]. You need to construct your own [`hyper]` client and
    /// use it together with the IP address and bearer token for the [`Hub`].
    pub fn new(
        client: hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
        ip_address: std::net::Ipv4Addr,
        token: String,
    ) -> Self {
        Hub {
            client,
            ip_address,
            token,
        }
    }

    fn create_request(
        &self,
        method: http::Method,
        path: &str,
        body: Option<hyper::Body>,
    ) -> anyhow::Result<http::Request<hyper::Body>> {
        let uri: hyper::Uri = format!(
            "https://{}:{}/{}{}",
            self.ip_address, DIRIGERA_PORT, DIRIGERA_API_VERSION, path,
        )
        .try_into()?;

        let request = http::Request::builder()
            .method(method)
            .uri(&uri)
            .header(http::header::CONTENT_TYPE, "application/json")
            .header("User-Agent", "dirigera-rs/0.1.0")
            .header("Authorization", format!("Bearer {}", self.token));

        let req = match body {
            Some(body) => request.body(body),
            None => request.body(hyper::Body::empty()),
        };

        req.map_err(|err| anyhow::anyhow!(err))
    }

    async fn deserialize_response<T>(response: http::Response<hyper::Body>) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let (_, body) = response.into_parts();
        let body = hyper::body::to_bytes(body).await?;

        serde_json::from_slice(body.as_ref()).map_err(|err| anyhow::anyhow!(err))
    }

    /// List all devices that is known for the [`Hub`]. This will return an exhaustive list of
    /// [`Device`](crate::Device)s.
    pub async fn devices(&mut self) -> anyhow::Result<Vec<crate::Device>> {
        Self::deserialize_response(
            self.client
                .call(self.create_request(http::Method::GET, "/devices", None)?)
                .await?,
        )
        .await
    }

    /// Get a single [`Device`](crate::Device) based on its id.
    pub async fn device(&mut self, id: &str) -> anyhow::Result<crate::Device> {
        Self::deserialize_response(
            self.client
                .call(self.create_request(
                    http::Method::GET,
                    format!("/devices/{}", id).as_str(),
                    None,
                )?)
                .await?,
        )
        .await
    }

    /// Rename a [`Device`](crate::Device). The function takes a mutable reference to the
    /// [`Device`](crate::Device) because on successful renaming the passed
    /// [`Device`](crate::Device) will be updated with the new name.
    pub async fn rename(
        &mut self,
        device: &mut crate::device::Device,
        new_name: &str,
    ) -> anyhow::Result<()> {
        let inner = device.inner_mut();

        if !has_capability(
            inner.capabilities.can_receive.as_ref(),
            &[crate::device::Capability::CustomName],
        ) {
            anyhow::bail!("device cannot change name");
        }

        let mut attributes = HashMap::new();
        attributes.insert("customName", new_name);

        let mut body = HashMap::new();
        body.insert("attributes", attributes);

        let body: String = serde_json::to_string(&vec![body])?;

        self.client
            .call(self.create_request(
                http::Method::PATCH,
                format!("/devices/{}", inner.id).as_str(),
                Some(hyper::Body::from(body)),
            )?)
            .await?;

        inner.attributes.custom_name = new_name.to_string();

        Ok(())
    }

    /// Toggle a [`Device`](crate::Device) on and off. Requires the [`Device`](crate::Device) to
    /// support [`Capability::IsOn`](crate::device::Capability::IsOn) as a receivable capability.
    /// The function takes a mutable reference to the [`Device`](crate::Device) because on
    /// successful toggle the passed
    /// [`Device`](crate::Device) will be updated with the new state.
    pub async fn toggle_on_off(
        &mut self,
        device: &mut crate::device::Device,
    ) -> anyhow::Result<()> {
        let inner = device.inner_mut();

        if !has_capability(
            inner.capabilities.can_receive.as_ref(),
            &[crate::device::Capability::IsOn],
        ) {
            anyhow::bail!("device cannot be toggled");
        }

        let mut attributes = HashMap::new();

        inner.attributes.is_on.map(|x| attributes.insert("isOn", !x));

        let mut body = HashMap::new();
        body.insert("attributes", attributes);

        let body: String = serde_json::to_string(&vec![body])?;

        self.client
            .call(self.create_request(
                http::Method::PATCH,
                format!("/devices/{}", inner.id).as_str(),
                Some(hyper::Body::from(body)),
            )?)
            .await?;

        inner.attributes.is_on = inner.attributes.is_on.map(|x| !x);

        Ok(())
    }

    /// Set light level on the [`Device`](crate::Device). Requires the [`Device`](crate::Device) to
    /// support [`Capability::LightLevel`](crate::device::Capability::LightLevel) as a receivable
    /// capability. The function takes a mutable reference to the [`Device`](crate::Device) because
    /// on successful change the passed [`Device`](crate::Device) will be updated with the new
    /// light level.
    pub async fn set_light_level(
        &mut self,
        device: &mut crate::device::Device,
        level: i8,
    ) -> anyhow::Result<()> {
        let inner = device.inner_mut();

        if !has_capability(
            inner.capabilities.can_receive.as_ref(),
            &[crate::device::Capability::LightLevel],
        ) {
            anyhow::bail!("device cannot set light level");
        }

        if level > 100 {
            anyhow::bail!("level must be between 0.0 -> 100.0");
        }

        let mut attributes = HashMap::new();
        attributes.insert("lightLevel", level);

        let mut body = HashMap::new();
        body.insert("attributes", attributes);

        let body: String = serde_json::to_string(&vec![body])?;

        self.client
            .call(self.create_request(
                http::Method::PATCH,
                format!("/devices/{}", inner.id).as_str(),
                Some(hyper::Body::from(body)),
            )?)
            .await?;

        inner.attributes.light_level = Some(level);

        Ok(())
    }

    /// Set color temperature on the [`Device`](crate::Device). Requires the
    /// [`Device`](crate::Device) to support
    /// [`Capability::ColorTemperature`](crate::device::Capability::ColorTemperature) as a
    /// receivable capability. The function takes a mutable reference to the
    /// [`Device`](crate::Device) because on successful change the passed [`Device`](crate::Device)
    /// will be updated with the new color temperature.
    pub async fn set_temperature(
        &mut self,
        device: &mut crate::device::Device,
        temperature: u16,
    ) -> anyhow::Result<()> {
        let inner = device.inner_mut();

        if !has_capability(
            inner.capabilities.can_receive.as_ref(),
            &[crate::device::Capability::ColorTemperature],
        ) {
            anyhow::bail!("device cannot set color temperature");
        }

        let min = inner
            .attributes
            .color_temperature_min
            .ok_or_else(|| anyhow::anyhow!("device has no min temperature value"))?;
        let max = inner
            .attributes
            .color_temperature_max
            .ok_or_else(|| anyhow::anyhow!("device has no max temperature value"))?;

        if !(max..=min).contains(&temperature) {
            anyhow::bail!("color temperature {temperature} not within {min} -> {max}");
        }

        let mut attributes = HashMap::new();
        attributes.insert("colorTemperature", temperature);

        let mut body = HashMap::new();
        body.insert("attributes", attributes);

        let body: String = serde_json::to_string(&vec![body])?;

        self.client
            .call(self.create_request(
                http::Method::PATCH,
                format!("/devices/{}", inner.id).as_str(),
                Some(hyper::Body::from(body)),
            )?)
            .await?;

        inner.attributes.color_temperature = Some(temperature);

        Ok(())
    }

    /// Set hue and saturation on the [`Device`](crate::Device). Requires the
    /// [`Device`](crate::Device) to support
    /// [`Capability::ColorHue`](crate::device::Capability::ColorHue) and
    /// [`Capability::ColorSaturation`](crate::device::Capability::ColorSaturation) as a receivable
    /// capability. The function takes a mutable reference to the [`Device`](crate::Device) because
    /// on successful change the passed [`Device`](crate::Device) will be updated with the new hue
    /// and saturation.
    pub async fn set_hue_saturation(
        &mut self,
        device: &mut crate::device::Device,
        hue: f64,
        saturation: f64,
    ) -> anyhow::Result<()> {
        let inner = device.inner_mut();

        if !has_capability(
            inner.capabilities.can_receive.as_ref(),
            &[
                crate::device::Capability::ColorHue,
                crate::device::Capability::ColorSaturation,
            ],
        ) {
            anyhow::bail!("device cannot be change for hue and saturation");
        }

        if !(0f64..=360f64).contains(&hue) {
            anyhow::bail!("hue must be between 0.0 -> 360.0");
        }

        if !(0f64..=1f64).contains(&saturation) {
            anyhow::bail!("hue must be between 0.0 -> 1.0");
        }

        let mut attributes = HashMap::new();
        attributes.insert("colorHue", hue);
        attributes.insert("colorSaturation", saturation);

        let mut body = HashMap::new();
        body.insert("attributes", attributes);

        let body: String = serde_json::to_string(&vec![body])?;

        self.client
            .call(self.create_request(
                http::Method::PATCH,
                format!("/devices/{}", inner.id).as_str(),
                Some(hyper::Body::from(body)),
            )?)
            .await?;

        inner.attributes.color_hue = Some(hue);
        inner.attributes.color_saturation = Some(hue);

        Ok(())
    }

    /// Set startup behaviour on the [`Device`](crate::Device). The function takes a mutable
    /// reference to the [`Device`](crate::Device) because on successful change the passed
    /// [`Device`](crate::Device) will be updated with the new startup behaviour.
    pub async fn set_startup_behaviour(
        &mut self,
        device: &mut crate::device::Device,
        behaviour: crate::device::Startup,
    ) -> anyhow::Result<()> {
        let inner = device.inner_mut();

        let mut attributes = HashMap::new();
        attributes.insert("startupOnOff", &behaviour);

        let mut body = HashMap::new();
        body.insert("attributes", attributes);

        let body: String = serde_json::to_string(&vec![body])?;

        self.client
            .call(self.create_request(
                http::Method::PATCH,
                format!("/devices/{}", inner.id).as_str(),
                Some(hyper::Body::from(body)),
            )?)
            .await?;

        inner.attributes.startup_on_off = Some(behaviour);

        Ok(())
    }

    /// Set target level on the [`Device`](crate::Device). Requires the [`Device`](crate::Device)
    /// to support [`Capability::BlindsState`](crate::device::Capability::BlindsState) as a
    /// receivable capability. The function takes a mutable reference to the
    /// [`Device`](crate::Device) because on successful change the passed [`Device`](crate::Device)
    /// will be updated with the new target level for the blinds.
    pub async fn set_target_level(
        &mut self,
        device: &mut crate::device::Device,
        level: u8,
    ) -> anyhow::Result<()> {
        let inner = device.inner_mut();

        if !has_capability(
            inner.capabilities.can_receive.as_ref(),
            &[crate::device::Capability::BlindsState],
        ) {
            anyhow::bail!("device cannot be change for blind state");
        }

        if level > 100 {
            anyhow::bail!("level must be between 0.0 -> 100.0");
        }

        let mut attributes = HashMap::new();
        attributes.insert("blindsTargetLevel", level);

        let mut body = HashMap::new();
        body.insert("attributes", attributes);

        let body: String = serde_json::to_string(&vec![body])?;

        self.client
            .call(self.create_request(
                http::Method::PATCH,
                format!("/devices/{}", inner.id).as_str(),
                Some(hyper::Body::from(body)),
            )?)
            .await?;

        inner.attributes.blinds_target_level = Some(level);

        Ok(())
    }

    /// List all scenes that is known for the [`Hub`]. This will return an exhaustive list of
    /// [`Scene`](crate::Scene)s.
    pub async fn scenes(&mut self) -> anyhow::Result<Vec<crate::Scene>> {
        Self::deserialize_response(
            self.client
                .call(self.create_request(http::Method::GET, "/scenes", None)?)
                .await?,
        )
        .await
    }

    /// Get a single [`Scene`](crate::Scene) based on its id.
    pub async fn scene(&mut self, id: &str) -> anyhow::Result<crate::Scene> {
        Self::deserialize_response(
            self.client
                .call(self.create_request(
                    http::Method::GET,
                    format!("/scenes/{}", id).as_str(),
                    None,
                )?)
                .await?,
        )
        .await
    }

    /// Trigger a [`Scene`](crate::Scene) now. Will work independent of a scheduled scene or not.
    pub async fn trigger_scene(&mut self, scene: &crate::scene::Scene) -> anyhow::Result<()> {
        let inner = scene.inner();

        self.client
            .call(self.create_request(
                http::Method::POST,
                format!("/scenes/{}/trigger", inner.id).as_str(),
                Some(hyper::Body::empty()),
            )?)
            .await?;

        Ok(())
    }

    /// Undo scene will revert the changes set by the [`Scene`](crate::Scene).
    pub async fn undo_scene(&mut self, scene: &crate::scene::Scene) -> anyhow::Result<()> {
        let inner = scene.inner();

        self.client
            .call(self.create_request(
                http::Method::POST,
                format!("/scenes/{}/undo", inner.id).as_str(),
                Some(hyper::Body::empty()),
            )?)
            .await?;

        Ok(())
    }
}

fn has_capability(
    got: &[crate::device::Capability],
    required: &[crate::device::Capability],
) -> bool {
    required.iter().all(|item| got.contains(item))
}
