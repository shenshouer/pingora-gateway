use std::{
    fmt::Debug,
    future::{ready, Future},
    hash::Hash,
    pin::Pin,
};

use futures::{FutureExt, StreamExt};
use k8s_openapi::serde::de::DeserializeOwned;
use kube::{
    runtime::{
        self,
        reflector::{self, Lookup, Store},
        WatchStreamExt,
    },
    Api, Client, Resource,
};
use tracing::{error, info};

// type WatchEventStream<K> =
//     Mutex<Option<Pin<Box<dyn Stream<Item = Result<watcher::Event<K>, watcher::Error>> + Send>>>>;

type WatchEventFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

pub fn watch<K>(client: Client) -> (Store<K>, WatchEventFuture)
where
    K: Resource + Clone + Send + Sync + DeserializeOwned + Debug + 'static,
    K: Lookup<DynamicType = <K as Resource>::DynamicType>,
    <K as Resource>::DynamicType: Eq + Hash + Clone + Default,
{
    let api = Api::<K>::all(client.clone());
    let (store, writer) = reflector::store();
    let watcher = runtime::watcher(api, Default::default());
    let watcher_stream = watcher
        .default_backoff()
        .reflect(writer)
        .for_each(|r| {
            match r {
                Ok(e) => info!("Received event {e:?}"),
                Err(e) => error!("Error in watcher stream: {e}"),
            }
            ready(())
        })
        .boxed();
    (store, watcher_stream)
}
