// Additional XMPP stanza handlers and protocol implementations
use tokio_xmpp::Element;
use xmpp_parsers::{
    message::Message,
    presence::Presence,
    iq::Iq,
    disco::{DiscoInfoResult, DiscoItemsResult, Info, Item},
    version::VersionResult,
    ping::Ping,
    data_forms::DataForm,
    muc::MucUser,
};
use crate::xmpp::{XmppEvent, ns};
use crate::storage::Database;
use tokio::sync::broadcast;

pub struct StanzaHandler {
    event_tx: broadcast::Sender<XmppEvent>,
    database: Arc<Database>,
}

impl StanzaHandler {
    pub fn new(
        event_tx: broadcast::Sender<XmppEvent>,
        database: Arc<Database>,
    ) -> Self {
        Self {
            event_tx,
            database,
        }
    }

    pub async fn handle_service_discovery_info(
        &self,
        from: xmpp_parsers::Jid,
        info: DiscoInfoResult,
    ) {
        let identities = info.identities.into_iter()
            .map(|id| crate::xmpp::events::ServiceIdentity {
                category: id.category,
                type_name: id.type_,
                name: id.name,
                lang: id.lang,
            })
            .collect();

        let _ = self.event_tx.send(XmppEvent::DiscoInfoReceived {
            from,
            identities,
            features: info.features,
        });
    }

    pub async fn handle_service_discovery_items(
        &self,
        from: xmpp_parsers::Jid,
        items: DiscoItemsResult,
    ) {
        let disco_items = items.items.into_iter()
            .map(|item| crate::xmpp::events::DiscoItem {
                jid: item.jid,
                name: item.name,
                node: item.node,
            })
            .collect();

        let _ = self.event_tx.send(XmppEvent::DiscoItemsReceived {
            from,
            items: disco_items,
        });
    }

    pub async fn handle_version_request(
        &self,
        from: xmpp_parsers::Jid,
        id: String,
    ) -> Option<Element> {
        let version_result = VersionResult {
            name: Some("XMPP Client".to_string()),
            version: Some("0.1.0".to_string()),
            os: Some(std::env::consts::OS.to_string()),
        };

        let iq = Iq::from_result(id, from, version_result);
        Some(iq.into())
    }

    pub async fn handle_ping_request(
        &self,
        from: xmpp_parsers::Jid,
        id: String,
    ) -> Option<Element> {
        let iq = Iq::from_result(id, from, ());
        Some(iq.into())
    }

    pub async fn handle_muc_user_presence(
        &self,
        presence: Presence,
        muc_user: MucUser,
    ) {
        // Handle MUC user presence (joins, leaves, role changes, etc.)
        for item in muc_user.items {
            if let (Some(from), Some(nick)) = (presence.from, item.nick) {
                if item.role.is_none() && item.affiliation.is_none() {
                    // User left the room
                    let _ = self.event_tx.send(XmppEvent::MucUserLeft {
                        room_jid: from.clone().with_resource(None),
                        nickname: nick.to_string(),
                    });
                } else {
                    // User joined or status changed
                    let _ = self.event_tx.send(XmppEvent::MucUserJoined {
                        room_jid: from.clone().with_resource(None),
                        nickname: nick.to_string(),
                        jid: item.jid,
                    });
                }
            }
        }
    }

    pub async fn handle_carbons_message(
        &self,
        message: Message,
        carbon_type: crate::xmpp::events::CarbonType,
    ) {
        if let Some(from) = message.from {
            if let Some(to) = message.to {
                let body = message.bodies.iter().next().map(|(_, body)| body.0.clone()).unwrap_or_default();
                let stanza_id = message.id.clone().unwrap_or_default();

                let _ = self.event_tx.send(XmppEvent::CarbonReceived {
                    carbon_type,
                    message: crate::xmpp::events::MessageInfo {
                        from,
                        to,
                        body,
                        stanza_id,
                        timestamp: Some(chrono::Utc::now()),
                    },
                });
            }
        }
    }

    pub async fn handle_delayed_message(
        &self,
        message: Message,
        delay_info: &crate::xmpp::stanza_handler::DelayInfo,
    ) {
        // Handle messages with XEP-0203 delays
        if let Some(from) = message.from {
            if let Some(to) = message.to {
                let body = message.bodies.iter().next().map(|(_, body)| body.0.clone()).unwrap_or_default();
                let stanza_id = message.id.clone().unwrap_or_default();

                let _ = self.event_tx.send(XmppEvent::MessageReceived {
                    from,
                    to,
                    body,
                    stanza_id,
                    timestamp: Some(delay_info.stamp),
                });
            }
        }
    }
}

pub struct DelayInfo {
    pub stamp: chrono::DateTime<chrono::Utc>,
    pub from: Option<xmpp_parsers::Jid>,
    pub reason: Option<String>,
}

impl DelayInfo {
    pub fn from_element(element: &Element) -> Option<Self> {
        if element.name() == "delay" && element.ns() == Some(ns::XEP_0203) {
            let stamp = element.attr("stamp")?;
            let stamp = chrono::DateTime::parse_from_rfc3339(stamp)
                .ok()?
                .with_timezone(&chrono::Utc);

            let from = element.attr("from")
                .and_then(|s| s.parse().ok());

            let reason = element.attr("reason")
                .map(|s| s.to_string());

            Some(Self {
                stamp,
                from,
                reason,
            })
        } else {
            None
        }
    }
}