use k8s_openapi::api::core::v1::Pod;
use kube::{api::ListParams, client::Client, Api};
use log::info;

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    let client = Client::try_default().await.unwrap();
    let api: Api<Pod> = Api::namespaced(client, "kube-system");
    api.list(&ListParams::default())
        .await
        .unwrap()
        .items
        .iter()
        .map(|pod| pod.metadata.name.as_ref().unwrap())
        .for_each(|name| info!("{}", name));
}
