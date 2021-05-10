use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use k8s_openapi::serde_json;
use kube::api::PostParams;
use kube::{
    api::{ListParams, WatchEvent},
    client::Client,
    Api, Error,
};
use log::{error, info};
use serde_json::json;

static NAMESPACE: &str = "rustoperator";
static SIDECAR_IMAGE: &str = "hazelcast/hazelcast:4.2";
static SIDECAR_NAME_PREFIX: &str = "hazelcast-";

#[tokio::main]
async fn main() -> Result<(), Error> {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    let client = Client::try_default().await.unwrap();
    let api: Api<Pod> = Api::namespaced(client, NAMESPACE);
    let mut stream = api.watch(&ListParams::default(), "0").await?.boxed();
    while let Some(event) = stream.try_next().await? {
        match event {
            WatchEvent::Added(pod) => {
                let namespace = pod.metadata.namespace.as_ref().unwrap();
                if namespace == NAMESPACE && !is_sidecar(&pod) {
                    if already_has_sidecar(&api, &pod).await {
                        info!("Sidecar already existing for pod {}", name_of(&pod))
                    } else {
                        create_sidecar(&api, &pod).await?;
                    }
                }
            }
            WatchEvent::Deleted(pod) => {
                if is_assigned_sidecar(&pod) {
                    let new_pod = Pod {
                        metadata: ObjectMeta {
                            resource_version: None,
                            ..pod.metadata.clone()
                        },
                        ..pod.clone()
                    };
                    api.create(&PostParams::default(), &new_pod).await?;
                }
                info!("DELETED: {}", name_of(&pod))
            }
            WatchEvent::Modified(pod) => info!("UPDATED: {}", name_of(&pod)),
            WatchEvent::Error(e) => error!("ERROR: {} {} ({})", e.code, e.message, e.status),
            _ => {}
        };
    }
    info!("Controller finished successfully");
    Ok(())
}

fn is_sidecar(pod: &Pod) -> bool {
    match pod.metadata.annotations.as_ref() {
        Some(annotations) => match annotations.get("sidecar") {
            Some(value) => value == "true",
            None => false,
        },
        None => false,
    }
}

async fn already_has_sidecar(api: &Api<Pod>, pod: &Pod) -> bool {
    api.list(&ListParams::default())
        .await
        .unwrap()
        .items
        .iter()
        .any(|p| name_of(pod) == SIDECAR_NAME_PREFIX.to_owned() + name_of(p))
}

fn is_assigned_sidecar(pod: &Pod) -> bool {
    name_of(pod).starts_with(SIDECAR_NAME_PREFIX)
        && pod.metadata.owner_references.is_some()
        && pod
            .metadata
            .owner_references
            .as_ref()
            .unwrap()
            .iter()
            .any(|owner| owner.kind == "Pod" && owner.api_version == "v1")
}

async fn create_sidecar(api: &Api<Pod>, owner: &Pod) -> Result<Pod, Error> {
    let owner_name = name_of(owner);
    let name = SIDECAR_NAME_PREFIX.to_owned() + owner_name;
    let manifest = json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": {
            "name": name,
            "namespace": NAMESPACE,
            "ownerReferences": [{
                  "apiVersion": "v1",
                  "kind": "Pod",
                  "name": owner_name,
                  "uid": owner.metadata.uid.as_ref().unwrap(),
              }],
            "labels": {"sidecar": "true"}
        },
        "spec": {
            "containers": [{
              "name": name,
              "image": SIDECAR_IMAGE,
            }],
        },
    });
    let pod: Pod = serde_json::from_value(manifest)?;
    api.create(&PostParams::default(), &pod).await
}

fn name_of(pod: &Pod) -> &str {
    pod.metadata.name.as_ref().unwrap().as_str()
}
