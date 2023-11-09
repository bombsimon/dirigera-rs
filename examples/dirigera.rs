#![allow(dead_code)]
use rand::Rng;

#[tokio::main]
async fn main() {
    let mut hub = dirigera::hub::Hub::default();

    trigger_scene(&mut hub).await.unwrap();
}

async fn get_devices(hub: &mut dirigera::hub::Hub) -> anyhow::Result<()> {
    for device in hub.devices().await? {
        let inner = device.inner();

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

async fn get_device(hub: &mut dirigera::hub::Hub) -> anyhow::Result<()> {
    println!(
        "{:#?}",
        hub.device("3b1a04db-9abe-4811-b60a-797970f51e8a_1").await?
    );

    Ok(())
}

async fn get_scenes(hub: &mut dirigera::hub::Hub) -> anyhow::Result<()> {
    for scene in hub.scenes().await? {
        println!("{:#?}\n", scene);
    }

    Ok(())
}

async fn toggle_light(hub: &mut dirigera::hub::Hub) -> anyhow::Result<()> {
    let mut light = hub.device("3b1a04db-9abe-4811-b60a-797970f51e8a_1").await?;

    loop {
        println!(
            "Light is on = {}, will toggle",
            light.inner().attributes.is_on
        );

        hub.toggle_on_off(&mut light).await?;
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }
}

async fn light_level(hub: &mut dirigera::hub::Hub) -> anyhow::Result<()> {
    let mut light = hub.device("3b1a04db-9abe-4811-b60a-797970f51e8a_1").await?;
    let mut level = 0;

    loop {
        level = if level <= 90 { level + 10 } else { 0 };
        println!(
            "Light has level = {}",
            light.inner().attributes.light_level.unwrap_or(0),
        );

        hub.set_light_level(&mut light, level).await?;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}

async fn temperature(hub: &mut dirigera::hub::Hub) -> anyhow::Result<()> {
    let mut light = hub.device("86a0fac2-d213-42bb-b1c1-00533ba468cd_1").await?;
    let mut temp = 2300;

    loop {
        temp = if temp <= 3900 { temp + 100 } else { 2300 };
        println!(
            "Light has temp = {}",
            light.inner().attributes.color_temperature.unwrap_or(0),
        );

        hub.set_temperature(&mut light, temp).await?;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}

async fn randomize_hue(hub: &mut dirigera::hub::Hub) -> anyhow::Result<()> {
    let mut light = hub.device("3b1a04db-9abe-4811-b60a-797970f51e8a_1").await?;
    if !light.inner().attributes.is_on {
        hub.toggle_on_off(&mut light).await?;
    }

    loop {
        println!(
            "Light has hue = {}, saturation = {}, will toggle",
            light.inner().attributes.color_hue.unwrap_or(0.0),
            light.inner().attributes.color_saturation.unwrap_or(0.0),
        );

        let mut rng = rand::thread_rng();

        let hue = rng.gen_range(0.0..360.0);
        let saturation = rng.gen_range(0.0..1.0);

        hub.set_hue_saturation(&mut light, hue, saturation).await?;

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

async fn startup(hub: &mut dirigera::hub::Hub) -> anyhow::Result<()> {
    let mut light = hub.device("3b1a04db-9abe-4811-b60a-797970f51e8a_1").await?;

    loop {
        println!(
            "Light startup is = {:?}",
            light.inner().attributes.startup_on_off,
        );

        let new_startup = match light.inner().attributes.startup_on_off {
            Some(dirigera::device::Startup::StartPrevious) => dirigera::device::Startup::StartOff,
            _ => dirigera::device::Startup::StartPrevious,
        };

        hub.set_startup_behaviour(&mut light, new_startup).await?;
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }
}

async fn trigger_scene(hub: &mut dirigera::hub::Hub) -> anyhow::Result<()> {
    let scene = hub.scene("744173bf-f7d6-4f27-9dee-d7a2345ffe00").await?;
    hub.trigger_scene(&scene).await?;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    hub.undo_scene(&scene).await?;

    let scene = hub.scene("744173bf-f7d6-4f27-9dee-d7a2345ffe00").await?;
    println!("{:#?}", scene);

    Ok(())
}
