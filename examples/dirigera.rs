use rand::Rng;

#[tokio::main]
async fn main() {
    let hub = dirigera::hub::Hub::default();

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    get_scenes(&hub, &client).await.unwrap();
}

#[allow(dead_code)]
async fn get_devices(hub: &dirigera::hub::Hub, client: &reqwest::Client) -> anyhow::Result<()> {
    let request = convert_request(client, hub.devices_request()?);
    let response = convert_response(client.execute(request).await?).await;
    let mut devices = hub.devices_parse_response(response)?;

    devices.sort_by(|a, b| {
        a.into_inner()
            .device_type
            .partial_cmp(&b.into_inner().device_type)
            .unwrap()
    });

    for device in devices {
        let inner = device.into_inner();

        println!(
            "{:<20} {:<40} {:<20} {}",
            inner.attributes.custom_name,
            inner.id,
            inner.device_type,
            inner
                .room
                .as_ref()
                .map(|room| room.name.clone())
                .unwrap_or("Unknown".to_string()),
        );
    }

    Ok(())
}

#[allow(dead_code)]
async fn get_scenes(hub: &dirigera::hub::Hub, client: &reqwest::Client) -> anyhow::Result<()> {
    let request = convert_request(client, hub.scenes_request()?);
    let response = convert_response(client.execute(request).await?).await;

    for scene in hub.scenes_parse_response(response)? {
        println!("{:#?}\n", scene);
    }

    Ok(())
}

#[allow(dead_code)]
async fn toggle_light(hub: &dirigera::hub::Hub, client: &reqwest::Client) -> anyhow::Result<()> {
    loop {
        let request = convert_request(
            client,
            hub.device_request("3b1a04db-9abe-4811-b60a-797970f51e8a_1")?,
        );
        let response = convert_response(client.execute(request).await?).await;
        let mut light = hub.device_parse_response(response)?;

        println!(
            "Light is on = {}, will toggle",
            light.into_inner().attributes.is_on
        );

        let request = convert_request(client, hub.device_toggle_request(&light)?);
        convert_response(client.execute(request).await?).await;

        let request = convert_request(client, hub.device_reload_request(&light)?);
        let response = convert_response(client.execute(request).await?).await;

        hub.device_reload_parse_response(&mut light, response)?;

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

#[allow(dead_code)]
async fn randomize_hue(hub: &dirigera::hub::Hub, client: &reqwest::Client) -> anyhow::Result<()> {
    loop {
        let request = convert_request(
            client,
            hub.device_request("3b1a04db-9abe-4811-b60a-797970f51e8a_1")?,
        );
        let response = convert_response(client.execute(request).await?).await;
        let mut light = hub.device_parse_response(response)?;

        println!(
            "Light has hue = {}, saturation = {}, will toggle",
            light.into_inner().attributes.color_hue.unwrap_or(0.0),
            light
                .into_inner()
                .attributes
                .color_saturation
                .unwrap_or(0.0),
        );

        let mut rng = rand::thread_rng();

        let hue = rng.gen_range(0.0..100.0);
        let saturation = rng.gen_range(0.0..1.0);

        let request = convert_request(
            client,
            hub.device_set_hue_saturation(&light, hue, saturation)?,
        );
        convert_response(client.execute(request).await?).await;

        let request = convert_request(client, hub.device_reload_request(&light)?);
        let response = convert_response(client.execute(request).await?).await;

        hub.device_reload_parse_response(&mut light, response)?;

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

fn convert_request<T>(client: &reqwest::Client, input: http::Request<T>) -> reqwest::Request
where
    T: serde::Serialize,
{
    let (parts, body) = input.into_parts();
    let uri = parts.uri.to_string();

    match parts.method {
        http::Method::GET => client.get(uri).headers(parts.headers).build().unwrap(),
        http::Method::PATCH => client
            .patch(uri)
            .headers(parts.headers)
            .json(&body)
            .build()
            .unwrap(),
        _ => todo!(),
    }
}

async fn convert_response(response: reqwest::Response) -> http::Response<bytes::Bytes> {
    let mut builder = http::Response::builder()
        .status(response.status())
        .version(response.version());

    let headers = builder.headers_mut().unwrap();

    headers.extend(
        response
            .headers()
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone())),
    );

    let buffer = response.bytes().await.unwrap();

    builder.body(buffer).unwrap()
}
