use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{ListParams, WatchEvent},
    client::Client,
    Api, Error,
};
use log::{error, info};

#[tokio::main]
async fn main() -> Result<(), Error> {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    let client = Client::try_default().await.unwrap();
    let api: Api<Pod> = Api::namespaced(client, "kube-system");
    let mut stream = api.watch(&ListParams::default(), "0").await?.boxed();
    while let Some(event) = stream.try_next().await? {
        match event {
            WatchEvent::Added(pod) => info!("ADDED: {}", name_of(&pod)),
            WatchEvent::Modified(pod) => info!("UPDATED: {}", name_of(&pod)),
            WatchEvent::Deleted(pod) => info!("DELETED: {}", name_of(&pod)),
            WatchEvent::Error(e) => error!("ERROR: {} {} ({})", e.code, e.message, e.status),
            _ => {}
        };
    }
    Ok(())
}

fn name_of(pod: &Pod) -> &str {
    pod.metadata.name.as_ref().unwrap().as_str()
}
