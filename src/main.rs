mod iroh_node;

use anyhow::Result;
use std::io::{Read};
use std::net::SocketAddr;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use iroh::bytes::Hash;
use iroh::sync::{Author, AuthorId, Namespace};
use tokio::sync::Notify;
use ctrlc;
use iroh::rpc_protocol::{AuthorImportRequest, ShareMode};
use crate::iroh_node::Iroh;
use futures::{Stream, StreamExt};
use iroh::sync_engine::LiveEvent;
use ed25519_dalek::{Signature, SignatureError, Signer, SigningKey, VerifyingKey};
use iroh::bytes::baomap::Store as BlobStore;
use iroh::sync::store::Store as DocStore;
use bytes::Bytes;


struct BulletinPost {
    timestamp: DateTime<Utc>,
    message: Hash,
}

struct Identity {
    author: AuthorId,
    display_name: String,
    bulletin: Vec<Bulletin>
}
struct Bulletin(DateTime<Utc>, Hash);

async fn start_node() -> Iroh  {
    let bind_addr: SocketAddr = format!("0.0.0.0:0").parse().unwrap();
    Iroh::new(bind_addr, Some(iroh::net::defaults::default_derp_map())).await.unwrap()
}


fn main() {
    let shutdown_notify = Arc::new(Notify::new());
    let handler_notify = shutdown_notify.clone();
    ctrlc::set_handler(move || {
        println!("Bailing!");
        handler_notify.notify_one();
    }).expect("Error setting ctrl-c handler");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        run(shutdown_notify).await;
    })

}


async fn run(shutdown_notify: Arc<Notify>) -> Result<()> {

    println!("Hello, world!");
    let node = start_node().await;
    let client = node.client();
    let docstore = node.docstore().unwrap();
    let blobstore = node.blobstore().unwrap();
    let value = "Hello world";
    let myhash = blobstore.import_bytes(Bytes::from(value)).await?;
    let a = Author::new(&mut rand::rngs::OsRng {});
    docstore.import_author(a.clone())?;
    println!("I just added a blob, its hash was {}",myhash);


    println!("made a key {:?}", a.public_key().as_bytes());


    let mut existing_authors = client.list_authors().await?;
    while let Some(foo) = existing_authors.next().await {
        println!("Found author with id {:?}", foo?.as_bytes());
    }

    let doc = docstore.new_replica(Namespace::new(&mut rand::rngs::OsRng {}))?;
    doc.insert("msg", &a, myhash.clone(), value.len() as u64);






    let a = client.create_author().await?;

    println!("author: {}",a);
    let d = client.create_doc().await?;
    d.set_bytes(a, b"hello".to_vec(), b"world".to_vec()).await?;


    let set_val = d.get_one(a, b"hello".to_vec()).await?.unwrap();
    let content_hash = set_val.content_hash();
    println!("content hash: {}", content_hash);

    let cbytes = d.get_content_bytes(content_hash).await?;
    let cvalue = String::from_utf8(cbytes.to_vec())?;
    println!("I got the value: {}", cvalue);

    let doctick = d.share(ShareMode::Write).await?;
    println!("and here's a ticket! {}", doctick);

    let mut events = d.subscribe().await?;


    let stupid_client = client.clone();
    let events_handle = tokio::spawn(async move {
        while let Some(Ok(event)) = events.next().await {
            match event {
                LiveEvent::InsertRemote {
                    from,
                    entry,
                    content_status: _content_status,
                } => {
                    println!("{} just added some content on {}", from, entry.namespace());
                },
                LiveEvent::ContentReady { hash } => {
                    println!("content at {} is ready!", hash);
                    let new_bytes = stupid_client.get_bytes(hash).await.unwrap();
                    println!("The data is {}", String::from_utf8(new_bytes.to_vec()).unwrap());
                }
                _ => {
                    println!("got some shit!! {:?}", event);
                }
            }

        }
    });

    println!("do i get here?");


    shutdown_notify.notified().await;
    events_handle.abort_handle(); //???
    println!("Shutting down thing");
    node.shutdown();

    Ok(())

}
