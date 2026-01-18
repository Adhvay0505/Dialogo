use crate::error::{XmppResult, XmppError};
use crate::xmpp::{XmppEvent, XmppClientConfig, create_message_jid, generate_message_id, generate_iq_id};
use crate::storage::Database;
use tokio::sync::{Mutex, broadcast, mpsc};
use tokio_xmpp::{AsyncClient, Packet, Element, ClientBuilder};
use xmpp_parsers::{
    Jid, message::Message, presence::Presence, iq::Iq,
    message::MessageType,
    presence::{Show as PresenceShow, Type as PresenceType},
    iq::IqType,
};
use std::sync::Arc;
use arc_swap::ArcSwap;
use futures::{StreamExt, SinkExt};

pub struct XmppClient {
    config: XmppClientConfig,
    client: Option<AsyncClient>,
    database: Arc<Database>,
    
    // State management
    state: Arc<ArcSwap<XmppClientState>>,
    event_tx: broadcast::Sender<XmppEvent>,
    command_tx: mpsc::Sender<XmppCommand>,
    
    // Runtime
    is_connected: Arc<Mutex<bool>>,
    reconnect_attempts: Arc<Mutex<u32>>,
}

#[derive(Debug, Clone)]
pub struct XmppClientConfig {
    pub jid: String,
    pub password: String,
    pub resource: String,
    pub server_host: String,
    pub server_port: u16,
    pub use_tls: bool,
    pub accept_invalid_certs: bool,
    pub auto_reconnect: bool,
    pub max_reconnect_attempts: u32,
    pub reconnect_delay: std::time::Duration,
}

impl Default for XmppClientConfig {
    fn default() -> Self {
        Self {
            jid: String::new(),
            password: String::new(),
            resource: "xmpp-client".to_string(),
            server_host: "localhost".to_string(),
            server_port: 5222,
            use_tls: true,
            accept_invalid_certs: false,
            auto_reconnect: true,
            max_reconnect_attempts: 5,
            reconnect_delay: std::time::Duration::from_secs(10),
        }
    }
}

#[derive(Debug, Clone)]
pub struct XmppClientState {
    pub connection_status: ConnectionStatus,
    pub authenticated: bool,
    pub roster: Vec<crate::storage::RosterItem>,
    pub connected_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for XmppClientState {
    fn default() -> Self {
        Self {
            connection_status: ConnectionStatus::Disconnected,
            authenticated: false,
            roster: Vec::new(),
            connected_at: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error(String),
}

#[derive(Debug)]
pub enum XmppCommand {
    Connect,
    Disconnect,
    SendMessage {
        to: Jid,
        body: String,
        chat_state: Option<ChatStateCommand>,
    },
    SendPresence {
        show: PresenceShow,
        status: Option<String>,
    },
    GetRoster,
    AddRosterItem {
        jid: Jid,
        name: Option<String>,
        groups: Vec<String>,
    },
    RemoveRosterItem {
        jid: Jid,
    },
    ApproveSubscription {
        jid: Jid,
    },
    DeclineSubscription {
        jid: Jid,
    },
    JoinMuc {
        room_jid: Jid,
        nickname: String,
        password: Option<String>,
    },
    LeaveMuc {
        room_jid: Jid,
    },
    SendMucMessage {
        room_jid: Jid,
        body: String,
    },
    SendFile {
        to: Jid,
        file_path: String,
    },
}

#[derive(Debug)]
pub enum ChatStateCommand {
    Active,
    Inactive,
    Gone,
    Composing,
    Paused,
}

impl XmppClient {
    pub fn new(
        config: XmppClientConfig,
        database: Arc<Database>,
        event_tx: broadcast::Sender<XmppEvent>,
    ) -> (Self, mpsc::Receiver<XmppCommand>) {
        let (command_tx, command_rx) = mpsc::channel(1000);
        
        let client = Self {
            config,
            client: None,
            database,
            state: Arc::new(ArcSwap::new(Arc::new(XmppClientState::default()))),
            event_tx,
            command_tx,
            is_connected: Arc::new(Mutex::new(false)),
            reconnect_attempts: Arc::new(Mutex::new(0)),
        };

        (client, command_rx)
    }

    pub fn get_state(&self) -> Arc<XmppClientState> {
        self.state.load().clone()
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<XmppEvent> {
        self.event_tx.subscribe()
    }

    pub async fn run(mut self, mut command_rx: mpsc::Receiver<XmppCommand>) -> XmppResult<()> {
        // Command processing loop
        while let Some(command) = command_rx.recv().await {
            match command {
                XmppCommand::Connect => {
                    if let Err(e) = self.connect().await {
                        let _ = self.event_tx.send(XmppEvent::ConnectionError {
                            error: e.to_string(),
                        });
                    }
                }
                XmppCommand::Disconnect => {
                    self.disconnect().await?;
                }
                XmppCommand::SendMessage { to, body, chat_state } => {
                    self.send_message(to, body, chat_state).await?;
                }
                XmppCommand::SendPresence { show, status } => {
                    self.send_presence(show, status).await?;
                }
                XmppCommand::GetRoster => {
                    self.request_roster().await?;
                }
                XmppCommand::AddRosterItem { jid, name, groups } => {
                    self.add_roster_item(jid, name, groups).await?;
                }
                XmppCommand::RemoveRosterItem { jid } => {
                    self.remove_roster_item(jid).await?;
                }
                XmppCommand::ApproveSubscription { jid } => {
                    self.approve_subscription(jid).await?;
                }
                XmppCommand::DeclineSubscription { jid } => {
                    self.decline_subscription(jid).await?;
                }
                XmppCommand::JoinMuc { room_jid, nickname, password } => {
                    self.join_muc(room_jid, nickname, password).await?;
                }
                XmppCommand::LeaveMuc { room_jid } => {
                    self.leave_muc(room_jid).await?;
                }
                XmppCommand::SendMucMessage { room_jid, body } => {
                    self.send_muc_message(room_jid, body).await?;
                }
                XmppCommand::SendFile { to, file_path } => {
                    self.send_file(to, file_path).await?;
                }
            }
        }

        Ok(())
    }

    async fn connect(&mut self) -> XmppResult<()> {
        let jid = create_message_jid(&self.config.jid, Some(&self.config.resource))?;
        
        self.update_state(|state| {
            state.connection_status = ConnectionStatus::Connecting;
        });

        let _ = self.event_tx.send(XmppEvent::Connecting);

        // Build XMPP client
        let mut builder = ClientBuilder::new(jid, &self.config.password)
            .set_server(&self.config.server_host, self.config.server_port);

        if self.config.use_tls {
            builder = builder.set_tls_insecure(self.config.accept_invalid_certs);
        }

        let (mut client, mut events) = builder.build().await?;

        // Send initial presence
        let presence = Presence::new(PresenceType::Available);
        let packet = Packet::Stanza(presence.into());
        client.send(packet).await?;

        // Update state
        self.client = Some(client.clone());
        *self.is_connected.lock().await = true;
        
        self.update_state(|state| {
            state.connection_status = ConnectionStatus::Connected;
            state.authenticated = true;
            state.connected_at = Some(chrono::Utc::now());
        });

        let _ = self.event_tx.send(XmppEvent::Connected { jid });
        let _ = self.event_tx.send(XmppEvent::AuthenticationSuccess);

        // Start event processing task
        let event_tx = self.event_tx.clone();
        let database = self.database.clone();
        let is_connected = self.is_connected.clone();
        
        tokio::spawn(async move {
            while let Some(event) = events.next().await {
                match event {
                    Ok(Packet::Stanza(stanza)) => {
                        Self::handle_stanza(stanza, &event_tx, &database).await;
                    }
                    Ok(Packet::Text(_)) => {
                        // Handle text packets if needed
                    }
                    Err(e) => {
                        let _ = event_tx.send(XmppEvent::Error {
                            error: format!("Stream error: {}", e),
                            stanza: None,
                        });
                    }
                }
            }
            
            *is_connected.lock().await = false;
        });

        Ok(())
    }

    async fn disconnect(&mut self) -> XmppResult<()> {
        if let Some(mut client) = self.client.take() {
            let presence = Presence::new(PresenceType::Unavailable);
            let packet = Packet::Stanza(presence.into());
            
            if let Err(e) = client.send(packet).await {
                tracing::warn!("Failed to send unavailable presence: {}", e);
            }

            if let Err(e) = client.end().await {
                tracing::warn!("Failed to close connection: {}", e);
            }
        }

        *self.is_connected.lock().await = false;
        self.update_state(|state| {
            state.connection_status = ConnectionStatus::Disconnected;
            state.authenticated = false;
        });

        let _ = self.event_tx.send(XmppEvent::Disconnected {
            reason: "User requested disconnect".to_string(),
        });

        Ok(())
    }

    async fn send_message(
        &self,
        to: Jid,
        body: String,
        chat_state: Option<ChatStateCommand>,
    ) -> XmppResult<()> {
        let stanza_id = generate_message_id();
        let from_jid = create_message_jid(&self.config.jid, Some(&self.config.resource))?;

        let mut message = Message::new(from_jid)
            .to(to.clone())
            .id(&stanza_id)
            .body(body.clone())
            .type_(MessageType::Chat);

        // Add chat state if specified
        if let Some(state) = chat_state {
            match state {
                ChatStateCommand::Active => {
                    message = message.active();
                }
                ChatStateCommand::Composing => {
                    message = message.composing();
                }
                ChatStateCommand::Paused => {
                    message = message.paused();
                }
                ChatStateCommand::Inactive => {
                    message = message.inactive();
                }
                ChatStateCommand::Gone => {
                    message = message.gone();
                }
            }
        }

        if let Some(client) = &self.client {
            client.send(Packet::Stanza(message.into())).await?;

            // Save to database
            let _ = self.database.save_message(
                &from_jid,
                &to,
                &body,
                "chat",
                &stanza_id,
            ).await;

            let _ = self.event_tx.send(XmppEvent::MessageSent {
                to,
                body,
                stanza_id,
            });
        }

        Ok(())
    }

    async fn send_presence(
        &self,
        show: PresenceShow,
        status: Option<String>,
    ) -> XmppResult<()> {
        let mut presence = Presence::new(PresenceType::Available).show(show);

        if let Some(status_text) = status {
            presence = presence.status(&status_text);
        }

        if let Some(client) = &self.client {
            client.send(Packet::Stanza(presence.into())).await?;

            let _ = self.event_tx.send(XmppEvent::PresenceSent {
                show: format!("{:?}", show),
                status,
            });
        }

        Ok(())
    }

    async fn request_roster(&self) -> XmppResult<()> {
        let from_jid = create_message_jid(&self.config.jid, Some(&self.config.resource))?;
        let iq_id = generate_iq_id();

        let iq = Iq::from_get(iq_id, from_jid)
            .with_payload(xmpp_parsers::roster::Roster::new());

        if let Some(client) = &self.client {
            client.send(Packet::Stanza(iq.into())).await?;
        }

        Ok(())
    }

    async fn add_roster_item(
        &self,
        jid: Jid,
        name: Option<String>,
        groups: Vec<String>,
    ) -> XmppResult<()> {
        let from_jid = create_message_jid(&self.config.jid, Some(&self.config.resource))?;
        let iq_id = generate_iq_id();

        let mut roster_item = xmpp_parsers::roster::Item::new(jid.clone());
        if let Some(item_name) = name {
            roster_item = roster_item.name(&item_name);
        }
        
        for group in groups {
            roster_item = roster_item.add_group(&group);
        }

        let roster = xmpp_parsers::roster::Roster::new().with_item(roster_item);
        let iq = Iq::from_set(iq_id, from_jid)
            .with_payload(roster);

        if let Some(client) = &self.client {
            client.send(Packet::Stanza(iq.into())).await?;

            let _ = self.database.add_roster_item(
                &create_message_jid(&self.config.jid, None)?,
                &jid,
                name.as_deref(),
                &groups,
            ).await;
        }

        Ok(())
    }

    async fn remove_roster_item(&self, jid: Jid) -> XmppResult<()> {
        let from_jid = create_message_jid(&self.config.jid, Some(&self.config.resource))?;
        let iq_id = generate_iq_id();

        let roster_item = xmpp_parsers::roster::Item::new(jid.clone()).subscription("remove");
        let roster = xmpp_parsers::roster::Roster::new().with_item(roster_item);
        let iq = Iq::from_set(iq_id, from_jid)
            .with_payload(roster);

        if let Some(client) = &self.client {
            client.send(Packet::Stanza(iq.into())).await?;
        }

        Ok(())
    }

    async fn approve_subscription(&self, jid: Jid) -> XmppResult<()> {
        let from_jid = create_message_jid(&self.config.jid, Some(&self.config.resource))?;
        let presence = Presence::new(PresenceType::Subscribed)
            .to(jid.clone())
            .from(from_jid);

        if let Some(client) = &self.client {
            client.send(Packet::Stanza(presence.into())).await?;

            let _ = self.event_tx.send(XmppEvent::SubscriptionApproved { jid });
        }

        Ok(())
    }

    async fn decline_subscription(&self, jid: Jid) -> XmppResult<()> {
        let from_jid = create_message_jid(&self.config.jid, Some(&self.config.resource))?;
        let presence = Presence::new(PresenceType::Unsubscribed)
            .to(jid.clone())
            .from(from_jid);

        if let Some(client) = &self.client {
            client.send(Packet::Stanza(presence.into())).await?;

            let _ = self.event_tx.send(XmppEvent::SubscriptionDeclined { jid });
        }

        Ok(())
    }

    async fn join_muc(
        &self,
        room_jid: Jid,
        nickname: String,
        password: Option<String>,
    ) -> XmppResult<()> {
        let from_jid = create_message_jid(&self.config.jid, Some(&self.config.resource))?;
        let full_jid = format!("{}/{}", room_jid, nickname);

        let mut presence = Presence::new(PresenceType::Available)
            .to(full_jid.parse().unwrap())
            .from(from_jid);

        // Add MUC namespace
        presence = presence.add_payload(
            Element::builder("x", xmpp_parsers::ns::MUC)
                .build()
        );

        if let Some(pwd) = password {
            let password_elem = Element::builder("password", xmpp_parsers::ns::MUC)
                .append(pwd)
                .build();
            presence = presence.add_payload(password_elem);
        }

        if let Some(client) = &self.client {
            client.send(Packet::Stanza(presence.into())).await?;

            let _ = self.event_tx.send(XmppEvent::MucJoined {
                room_jid,
                nickname,
            });
        }

        Ok(())
    }

    async fn leave_muc(&self, room_jid: Jid) -> XmppResult<()> {
        let from_jid = create_message_jid(&self.config.jid, Some(&self.config.resource))?;
        let presence = Presence::new(PresenceType::Unavailable)
            .to(room_jid)
            .from(from_jid);

        if let Some(client) = &self.client {
            client.send(Packet::Stanza(presence.into())).await?;

            let _ = self.event_tx.send(XmppEvent::MucLeft { room_jid });
        }

        Ok(())
    }

    async fn send_muc_message(&self, room_jid: Jid, body: String) -> XmppResult<()> {
        let stanza_id = generate_message_id();
        let from_jid = create_message_jid(&self.config.jid, Some(&self.config.resource))?;

        let message = Message::new(from_jid)
            .to(room_jid.clone())
            .id(&stanza_id)
            .body(body.clone())
            .type_(MessageType::Groupchat);

        if let Some(client) = &self.client {
            client.send(Packet::Stanza(message.into())).await?;

            let _ = self.event_tx.send(XmppEvent::MucMessageReceived {
                room_jid,
                from: from_jid,
                nickname: "me".to_string(),
                body,
                timestamp: Some(chrono::Utc::now()),
            });
        }

        Ok(())
    }

    async fn send_file(&self, to: Jid, file_path: String) -> XmppResult<()> {
        // This is a placeholder for file transfer implementation
        // In a real implementation, you would use XEP-0363 (HTTP File Upload)
        // or XEP-0096 (SI File Transfer)
        
        let _ = self.event_tx.send(XmppEvent::FileTransferError {
            transfer_id: generate_message_id(),
            error: "File transfer not yet implemented".to_string(),
        });

        Ok(())
    }

    async fn handle_stanza(
        stanza: Element,
        event_tx: &broadcast::Sender<XmppEvent>,
        database: &Arc<Database>,
    ) {
        if let Ok(message) = Message::try_from(stanza.clone()) {
            Self::handle_message(message, event_tx, database).await;
        } else if let Ok(presence) = Presence::try_from(stanza.clone()) {
            Self::handle_presence(presence, event_tx, database).await;
        } else if let Ok(iq) = Iq::try_from(stanza.clone()) {
            Self::handle_iq(iq, event_tx, database).await;
        }
    }

    async fn handle_message(
        message: Message,
        event_tx: &broadcast::Sender<XmppEvent>,
        database: &Arc<Database>,
    ) {
        if let Some(from) = message.from {
            let to = message.to.unwrap_or_else(|| from.clone());
            let body = message.bodies.iter().next().map(|(_, body)| body.0.clone()).unwrap_or_default();
            let stanza_id = message.id.clone().unwrap_or_default();

            // Save message to database
            let _ = database.save_message(
                &from,
                &to,
                &body,
                &format!("{:?}", message.type_),
                &stanza_id,
            ).await;

            // Check for chat states
            if message.composing.is_some() {
                let _ = event_tx.send(XmppEvent::ChatStateReceived {
                    from: from.clone(),
                    state: ChatState::Composing,
                });
            } else if message.active.is_some() {
                let _ = event_tx.send(XmppEvent::ChatStateReceived {
                    from: from.clone(),
                    state: ChatState::Active,
                });
            } else if message.paused.is_some() {
                let _ = event_tx.send(XmppEvent::ChatStateReceived {
                    from: from.clone(),
                    state: ChatState::Paused,
                });
            } else if message.inactive.is_some() {
                let _ = event_tx.send(XmppEvent::ChatStateReceived {
                    from: from.clone(),
                    state: ChatState::Inactive,
                });
            } else if message.gone.is_some() {
                let _ = event_tx.send(XmppEvent::ChatStateReceived {
                    from: from.clone(),
                    state: ChatState::Gone,
                });
            }

            // Send message received event if there's body content
            if !body.is_empty() {
                let _ = event_tx.send(XmppEvent::MessageReceived {
                    from,
                    to,
                    body,
                    stanza_id,
                    timestamp: Some(chrono::Utc::now()),
                });
            }
        }
    }

    async fn handle_presence(
        presence: Presence,
        event_tx: &broadcast::Sender<XmppEvent>,
        database: &Arc<Database>,
    ) {
        if let Some(from) = presence.from {
            let show = presence.show.map(|s| format!("{:?}", s)).unwrap_or("online".to_string());
            let status = presence.status.clone();
            let priority = presence.priority;

            // Update presence in database
            let _ = database.update_presence(
                &from,
                &show,
                status.as_deref(),
            ).await;

            // Handle subscription requests
            match presence.type_ {
                PresenceType::Subscribe => {
                    let _ = event_tx.send(XmppEvent::SubscriptionRequest { from });
                }
                PresenceType::Subscribed => {
                    let _ = event_tx.send(XmppEvent::SubscriptionApproved { from });
                }
                PresenceType::Unsubscribe => {
                    // Handle unsubscribe
                }
                PresenceType::Unsubscribed => {
                    let _ = event_tx.send(XmppEvent::SubscriptionDeclined { from });
                }
                PresenceType::Available | PresenceType::Unavailable => {
                    let _ = event_tx.send(XmppEvent::PresenceReceived {
                        from,
                        show,
                        status,
                        priority,
                    });
                }
                PresenceType::Error => {
                    // Handle error presence
                }
                PresenceType::Probe => {
                    // Handle presence probe
                }
            }

            // Handle MUC presence
            if from.node().is_some() && from.resource().is_some() {
                // This could be a MUC presence stanza
                // Additional MUC-specific handling would go here
            }
        }
    }

    async fn handle_iq(
        iq: Iq,
        event_tx: &broadcast::Sender<XmppEvent>,
        database: &Arc<Database>,
    ) {
        if let Ok(roster) = xmpp_parsers::roster::Roster::try_from(iq.payload.clone().unwrap()) {
            // Handle roster result
            let mut roster_items = Vec::new();
            let user_jid = iq.from.as_ref().unwrap_or(&iq.to.unwrap()).clone();

            for item in roster.items {
                let roster_item = crate::storage::RosterItem {
                    jid: item.jid.to_string(),
                    name: item.name.clone(),
                    subscription: item.subscription.to_string(),
                    groups: item.groups,
                    created_at: chrono::Utc::now(),
                };
                roster_items.push(roster_item.clone());

                // Save to database
                let _ = database.add_roster_item(
                    &user_jid,
                    &item.jid,
                    item.name.as_deref(),
                    &item.groups,
                ).await;
            }

            let _ = event_tx.send(XmppEvent::RosterReceived {
                items: roster_items.iter().map(|item| crate::xmpp::events::RosterItem {
                    jid: item.jid.parse().unwrap(),
                    name: item.name.clone(),
                    subscription: item.subscription.clone(),
                    groups: item.groups.clone(),
                    approved: false, // This would need to be parsed from the roster item
                    ask: None,
                }).collect(),
            });
        }
    }

    fn update_state<F>(&self, updater: F)
    where
        F: FnOnce(&mut XmppClientState),
    {
        let mut state = self.state.load().as_ref().clone();
        updater(&mut state);
        self.state.store(Arc::new(state));
    }
}