use serde::{Deserialize, Serialize};
use xmpp_parsers::{Jid, message::Message, presence::Presence, iq::Iq};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum XmppEvent {
    // Connection events
    Connected {
        jid: Jid,
    },
    Disconnected {
        reason: String,
    },
    Connecting,
    ConnectionError {
        error: String,
    },
    
    // Authentication events
    AuthenticationSuccess,
    AuthenticationError {
        error: String,
    },
    
    // Message events
    MessageReceived {
        from: Jid,
        to: Jid,
        body: String,
        stanza_id: String,
        timestamp: Option<DateTime<Utc>>,
    },
    MessageSent {
        to: Jid,
        body: String,
        stanza_id: String,
    },
    MessageDelivered {
        stanza_id: String,
    },
    MessageDisplayed {
        stanza_id: String,
    },
    
    // Chat state events
    ChatStateReceived {
        from: Jid,
        state: ChatState,
    },
    ChatStateChanged {
        to: Jid,
        state: ChatState,
    },
    
    // Presence events
    PresenceReceived {
        from: Jid,
        show: String,
        status: Option<String>,
        priority: Option<i32>,
    },
    PresenceSent {
        show: String,
        status: Option<String>,
    },
    
    // Roster events
    RosterReceived {
        items: Vec<RosterItem>,
    },
    RosterItemAdded {
        item: RosterItem,
    },
    RosterItemUpdated {
        item: RosterItem,
    },
    RosterItemRemoved {
        jid: Jid,
    },
    
    // Subscription events
    SubscriptionRequest {
        from: Jid,
    },
    SubscriptionApproved {
        jid: Jid,
    },
    SubscriptionDeclined {
        jid: Jid,
    },
    
    // MUC events
    MucJoined {
        room_jid: Jid,
        nickname: String,
    },
    MucLeft {
        room_jid: Jid,
    },
    MucMessageReceived {
        room_jid: Jid,
        from: Jid,
        nickname: String,
        body: String,
        timestamp: Option<DateTime<Utc>>,
    },
    MucSubjectChanged {
        room_jid: Jid,
        subject: String,
        changer: Option<Jid>,
    },
    MucUserJoined {
        room_jid: Jid,
        nickname: String,
        jid: Option<Jid>,
    },
    MucUserLeft {
        room_jid: Jid,
        nickname: String,
    },
    
    // File transfer events
    FileTransferRequest {
        from: Jid,
        filename: String,
        size: u64,
        mime_type: Option<String>,
        description: Option<String>,
    },
    FileTransferStarted {
        transfer_id: String,
        filename: String,
    },
    FileTransferProgress {
        transfer_id: String,
        progress: f64,
    },
    FileTransferCompleted {
        transfer_id: String,
        filename: String,
    },
    FileTransferError {
        transfer_id: String,
        error: String,
    },
    
    // Error events
    Error {
        error: String,
        stanza: Option<String>,
    },
    StanzaError {
        from: Jid,
        error_type: String,
        condition: String,
        text: Option<String>,
    },
    
    // Service Discovery events
    DiscoInfoReceived {
        from: Jid,
        identities: Vec<ServiceIdentity>,
        features: Vec<String>,
    },
    DiscoItemsReceived {
        from: Jid,
        items: Vec<DiscoItem>,
    },
    
    // Stream Management events
    StreamManagementEnabled {
        resume_id: Option<String>,
    },
    StreamManagementResumed {
        previously_received: u32,
    },
    StreamManagementFailed,
    
    // Carbons events
    CarbonReceived {
        carbon_type: CarbonType,
        message: MessageInfo,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatState {
    Active,
    Inactive,
    Gone,
    Composing,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosterItem {
    pub jid: Jid,
    pub name: Option<String>,
    pub subscription: String,
    pub groups: Vec<String>,
    pub approved: bool,
    pub ask: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceIdentity {
    pub category: String,
    pub type_name: String,
    pub name: Option<String>,
    pub lang: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoItem {
    pub jid: Jid,
    pub name: Option<String>,
    pub node: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CarbonType {
    Received,
    Sent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageInfo {
    pub from: Jid,
    pub to: Jid,
    pub body: String,
    pub stanza_id: String,
    pub timestamp: Option<DateTime<Utc>>,
}