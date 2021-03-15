use std::{
    time::Duration,
    error::Error,
    task::{Context, Poll},
    collections::hash_map::DefaultHasher,
    hash::{
        Hash,
        Hasher,
    },
};
use env_logger::{Builder, Env};
use futures::prelude::*;
use libp2p::{
    identity,
    PeerId,
    gossipsub,
    gossipsub::{
        protocol::MessageId,
        GossipsubEvent,
        GossipsubMessage,
        Topic,
    },
};
use async_std::{io, task};

fn main() -> Result<(), Box<dyn Error>>{
    Builder::from_env(Env::default().default_filter_or("info")).init();
    // Generate key for local node
    let node_key = identity::Keypair::generate_ed25519();
    let node_peer_id = PeerId::from_public_key(node_key.public());
    println!("PeerId: {:?}", node_peer_id);

    //development transport is tcp over mplex and yamux;
    let transport = libp2p::build_development_transport(node_key)?;
    let topic = Topic::new(String::from("chitter"));
    
    let mut swarm = {
        // Use default hasher to make messages content-addressable
        let gen_message_id = |message: &GossipsubMessage| {
            let mut hasher = DefaultHasher::new();
            message.data.hash(&mut hasher);
            MessageId(hasher.finish().to_string())
        };

        let gossipsub_config = gossipsub::GossipsubConfigBuilder::new()
            .heartbeat_interval(Duration::from_secs(5))
            .message_id_fn(gen_message_id)
            .build();
        
        let mut gossipsub = gossipsub::Gossipsub::new(node_peer_id.clone(), gossipsub_config);
        gossipsub.subscribe(topic.clone());
        libp2p::Swarm::new(transport, gossipsub, node_peer_id)
    };

    libp2p::Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse().unwrap())
        .expect("Failed to start listener on multiaddr");

    if let Some(to_dial) = std::env::args().nth(1) {
        let address = to_dial.clone();
        match to_dial.parse() {
            Ok(to_dial) => match libp2p::Swarm::dial_addr(&mut swarm, to_dial) {
                Ok(_) => println!("Dialed {:?}", address),
                Err(e) => println!("Failed to dial {:?}!\n Error: {:?}", address, e)
            },
            Err(e) => println!("Failed to parse address to dial {:?}!\n Error: {:?}", address, e)
        }
    }

    let mut stdin = io::BufReader::new(io::stdin()).lines();
    let mut listening = false;

    task::block_on(future::poll_fn(move |cx: &mut Context| {
        loop {
          match stdin.try_poll_next_unpin(cx)? {
            Poll::Ready(Some(line)) => swarm.publish(&topic, line.as_bytes()),
            Poll::Ready(None) => panic!("Stdin closed!"),
            Poll::Pending => break,
          }
        }
        loop {
            match swarm.poll_next_unpin(cx) {
                Poll::Ready(Some(gossip_event)) => match gossip_event {
                    GossipsubEvent::Message(peer_id, message_id, message) => println!(
                        "Incoming message {:?} from {:?}\n >{:?}", 
                        message_id, 
                        peer_id, 
                        String::from_utf8_lossy(&message.data),
                        ),
                    _ => {},
                },
                _ => break,
                }
            }
        if !listening {
            for address in libp2p::Swarm::listeners(&swarm) {
                println!("Listening on {:?}", address);
                listening = true;
            }
        }

        Poll::Pending
    }))
}
