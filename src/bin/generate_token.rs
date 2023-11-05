#[cfg(feature = "binary")]
use std::collections::HashMap;
use std::io::Write;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let ip_address = if args.len() < 2 {
        print!("Enter ip address: ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        input.trim().to_string()
    } else {
        args[1].to_string()
    };

    let file_path = "config.toml";
    if std::path::Path::new(file_path).exists() {
        anyhow::bail!("'config.toml' already exist!");
    }

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let code_verify = pkce::code_verifier(128);
    let code_challenge = pkce::code_challenge(&code_verify);
    let code_veirifier_string = String::from_utf8(code_verify)?;

    let mut authorize_params = HashMap::new();
    authorize_params.insert("audience", "homesmart.local");
    authorize_params.insert("response_type", "code");
    authorize_params.insert("code_challenge", code_challenge.as_str());
    authorize_params.insert("code_challenge_method", "S256");

    let mut auth_url =
        url::Url::parse(format!("https://{}:8443/v1/oauth/authorize", ip_address).as_str())?;
    for (key, value) in authorize_params.iter() {
        auth_url.query_pairs_mut().append_pair(key, value);
    }

    let response: HashMap<String, String> = client.get(auth_url).send().await?.json().await?;
    let code = response
        .get("code")
        .ok_or_else(|| anyhow::anyhow!("code not found in body: {:?}", response))?;

    println!("Press ENTER after pressing the button on your Dirigera device");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let mut token_params = HashMap::new();
    token_params.insert("code", code.as_str());
    token_params.insert("name", "localhost");
    token_params.insert("grant_type", "authorization_code");
    token_params.insert("code_verifier", code_veirifier_string.as_str());

    let token_url =
        url::Url::parse(format!("https://{}:8443/v1/oauth/token", ip_address).as_str())?;

    let response: HashMap<String, String> = client
        .post(token_url)
        .json(&token_params)
        .send()
        .await?
        .json()
        .await?;

    let access_token = response
        .get("access_token")
        .ok_or_else(|| anyhow::anyhow!("code not found in body: {:?}", response))?;

    let mut config = toml::value::Table::new();
    config.insert(
        "ip-address".to_string(),
        toml::Value::String(ip_address.to_string()),
    );
    config.insert(
        "token".to_string(),
        toml::Value::String(access_token.to_string()),
    );

    let mut file = std::fs::File::create(file_path)?;
    let toml_string = toml::to_string(&config)?;
    file.write_all(toml_string.as_bytes())?;

    println!("🎉 Configuration has been saved to 'config.toml'");

    Ok(())
}
