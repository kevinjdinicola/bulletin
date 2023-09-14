use anyhow::Result;
use iroh::node::Node;
use iroh::baomap::{flat::Store as BaoFileStore, mem::Store as BaoMemStore};
use iroh::bytes::util::runtime;
use iroh::sync::store::fs::Store as DocFileStore;
use iroh::sync::store::memory::Store as DocMemStore;
use std::net::SocketAddr;
use iroh::net::derp::DerpMap;
use iroh::rpc_protocol::{AuthorImportRequest, ProviderService};
use iroh::sync::Author;
use iroh::sync::store::Store;

use ed25519_dalek::{Signature, SignatureError, Signer, SigningKey, VerifyingKey};
use iroh::client::mem::RpcClient;


pub enum Iroh {
    FileStore( Node<BaoFileStore, DocFileStore>, BaoFileStore, DocFileStore ),
    MemStore( Node<BaoMemStore, DocMemStore>, BaoMemStore, DocMemStore ),
}

// im
impl Iroh {
    pub async fn new(bind_addr: SocketAddr, derp_map: Option<DerpMap>) -> Result<Self> {
        let rt = runtime::Handle::from_current(1)?;
        let node_rt = rt.clone();
        let db = BaoMemStore::new(rt);
        let doc_store = DocMemStore::default();
        let mut node = Node::builder(db.clone(), doc_store.clone())
            .bind_addr(bind_addr)
            .runtime(&node_rt);

        if let Some(derp) = derp_map {
            node = node.enable_derp(derp);
        }

       let node =  node
           .spawn()
           .await?;

        Ok(Iroh::MemStore(node, db, doc_store))
    }

    pub fn controller(&self) -> RpcClient
    {
        match self {
            Iroh::FileStore(node, _, _) => node.controller(),
            Iroh::MemStore(node, _, _) => node.controller(),
        }
    }

    pub fn blobstore(&self) -> Option<BaoMemStore> {
        match self {
            Iroh::FileStore(_, _, _) => None,
            Iroh::MemStore(_, bao, _) => Some(bao.clone()),
        }
    }


    pub fn docstore(&self) -> Option<DocMemStore> {
        match self {
            Iroh::FileStore(_, _, doc) => None,
            Iroh::MemStore(_, _, doc) => Some(doc.clone()),
        }
    }

    pub fn client(&self) -> iroh::client::mem::Iroh {
        match self {
            Iroh::FileStore(node, _, _) => node.client(),
            Iroh::MemStore(node, _, _) => node.client(),
        }
    }


    pub fn shutdown(self) {
        match self {
            Iroh::FileStore(node,_,_) => node.shutdown(),
            Iroh::MemStore(node,_,_) => node.shutdown(),
        }
    }
}