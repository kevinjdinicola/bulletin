use anyhow::Result;
use iroh::node::Node;
use iroh::baomap::{flat::Store as BaoFileStore, mem::Store as BaoMemStore};
use iroh::bytes::util::runtime;
use iroh::sync::store::fs::Store as DocFileStore;
use iroh::sync::store::memory::Store as DocMemStore;
use std::net::SocketAddr;
use quic_rpc::RpcClient;
use iroh::net::derp::DerpMap;
use iroh::rpc_protocol::ProviderService;
use iroh::sync::store::Store;


pub enum Iroh {
    FileStore( Node<BaoFileStore, DocFileStore> ),
    MemStore( Node<BaoMemStore, DocMemStore> ),
}


impl Iroh {
    pub async fn new(bind_addr: SocketAddr, derp_map: Option<DerpMap>) -> Result<Self> {
        let rt = runtime::Handle::from_current(1)?;
        let node_rt = rt.clone();
        let db = BaoMemStore::new(rt);
        let doc_store = DocMemStore::default();
        let mut node = Node::builder(db, doc_store)
            .bind_addr(bind_addr)
            .runtime(&node_rt);

        if let Some(derp) = derp_map {
            node = node.enable_derp(derp);
        }

       let node =  node
           .spawn()
           .await?;

        Ok(Iroh::MemStore(node))
    }

    // typing in rust is almost as bad as typescript
    // pub fn node(&self) -> &Node<D: Map, S: DocStore> {
    //     match self {
    //         Iroh::FileStore(node) => node,
    //         Iroh::MemStore(node) => node,
    //     }
    // }

    pub fn client(&self) -> iroh::client::mem::Iroh {
        match self {
            Iroh::FileStore(node) => node.client(),
            Iroh::MemStore(node) => node.client(),
        }
    }


    pub fn shutdown(self) {
        match self {
            Iroh::FileStore(node) => node.shutdown(),
            Iroh::MemStore(node) => node.shutdown(),
        }
    }
}