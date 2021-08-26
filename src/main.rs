use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::Metadata;
use kube::error::Error;
use kube::{api::ListParams, client::Client, Api};
use log::info;

#[tokio::main]
async fn main() -> Result<(), Error> {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    let client = Client::try_default().await.unwrap();
    let api: Api<Pod> = Api::namespaced(client, "kube-system");
    for p in api.list(&ListParams::default()).await? {
        info!("Found Pod: {}", p.metadata().name.as_ref().unwrap());
    }
    Ok(())
}
