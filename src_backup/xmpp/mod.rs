pub mod client;
pub mod events;
pub mod stanza_handler;

pub use client::XmppClient;
pub use events::*;

use tokio::sync::mpsc;
use tokio_xmpp::{AsyncClient, Packet, Element};
use xmpp_parsers::{Jid, message::Message, presence::Presence, iq::Iq};
use std::sync::Arc;
use arc_swap::ArcSwap;

// Core XMPP client state and event management
pub type XmppResult<T> = Result<T, crate::error::XmppError>;

// Constants for XMPP protocol
pub const DEFAULT_PORT: u16 = 5222;
pub const DEFAULT_RESOURCE: &str = "xmpp-client";

// XMPP Namespaces
pub mod ns {
    pub const CLIENT: &str = "jabber:client";
    pub const ROSTER: &str = "jabber:iq:roster";
    pub const MUC: &str = "http://jabber.org/protocol/muc";
    pub const MUC_USER: &str = "http://jabber.org/protocol/muc#user";
    pub const DISCO_INFO: &str = "http://jabber.org/protocol/disco#info";
    pub const DISCO_ITEMS: &str = "http://jabber.org/protocol/disco#items";
    pub const VERSION: &str = "jabber:iq:version";
    pub const TIME: &str = "jabber:iq:time";
    pub const PING: &str = "urn:xmpp:ping";
    pub const CHAT_STATES: &str = "http://jabber.org/protocol/chatstates";
    pub const XEP_0030: &str = "http://jabber.org/protocol/disco";
    pub const XEP_0045: &str = "http://jabber.org/protocol/muc";
    pub const XEP_0084: &str = "urn:xmpp:avatar:data";
    pub const XEP_0082: &str = "urn:xmpp:time";
    pub const XEP_0198: &str = "urn:xmpp:sm:3";
    pub const XEP_0199: &str = "urn:xmpp:ping";
    pub const XEP_0203: &str = "urn:xmpp:delay";
    pub const XEP_0224: &str = "urn:xmpp:hints";
    pub const XEP_0280: &str = "urn:xmpp:carbons:2";
    pub const XEP_0313: &str = "urn:xmpp:mam:2";
    pub const XEP_0352: &str = "urn:xmpp:csi:0";
    pub const XEP_0363: &str = "urn:xmpp:http:upload:0";
}

// Utility functions
pub fn create_message_jid(jid: &str, resource: Option<&str>) -> XmppResult<Jid> {
    let full_jid = if let Some(res) = resource {
        format!("{}/{}", jid, res)
    } else {
        jid.to_string()
    };
    
    Jid::from_str(&full_jid)
        .map_err(|e| crate::error::XmppError::InvalidJid(format!("Invalid JID: {}", e)))
}

pub fn extract_bare_jid(jid: &Jid) -> String {
    format!("{}@{}", jid.node().unwrap_or(""), jid.domain())
}

pub fn generate_message_id() -> String {
    use uuid::Uuid;
    format!("msg_{}", Uuid::new_v4())
}

pub fn generate_iq_id() -> String {
    use uuid::Uuid;
    format!("iq_{}", Uuid::new_v4())
}