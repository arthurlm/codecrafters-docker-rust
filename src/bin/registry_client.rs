use anyhow::Context;
use docker_starter_rust::registry::RegistryClient;
use tokio::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client =
        RegistryClient::auth("https://registry.hub.docker.com", "ubuntu", "latest").await?;

    let manifests = client.list_manifests().await?;
    let target_manifest = manifests
        .into_iter()
        .find(|m| m.platform.architecture == "amd64" && m.platform.os == "linux")
        .with_context(|| "No platform found")?;

    let image_manifest = client.read_image_manifest(&target_manifest).await?;
    let blob = client.read_blob(&image_manifest.layers[0]).await?;

    fs::write("data.tar.gz", &blob).await?;

    Ok(())
}
