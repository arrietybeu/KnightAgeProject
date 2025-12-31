use std::fmt;

/// Connection states - matching your Java implementation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionState {
    /// Just connected, waiting for key exchange
    Connected,
    /// Key exchange completed
    KeyExchanged,
    /// User authenticated (logged in)
    Authed,
    /// Player is in game
    InGame,
    /// Connection is closing
    Disconnecting,
}

impl ConnectionState {
    /// Check if current state is valid for a given set of allowed states
    pub fn is_valid_for(&self, allowed_states: &[ConnectionState]) -> bool {
        allowed_states.contains(self)
    }

    /// Get all states (for packets that work in any state)
    pub fn all() -> Vec<ConnectionState> {
        vec![
            ConnectionState::Connected,
            ConnectionState::KeyExchanged,
            ConnectionState::Authed,
            ConnectionState::InGame,
        ]
    }

    /// States after authentication
    pub fn authenticated() -> Vec<ConnectionState> {
        vec![ConnectionState::Authed, ConnectionState::InGame]
    }

    /// States before authentication
    pub fn pre_auth() -> Vec<ConnectionState> {
        vec![ConnectionState::Connected, ConnectionState::KeyExchanged]
    }
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionState::Connected => write!(f, "CONNECTED"),
            ConnectionState::KeyExchanged => write!(f, "KEY_EXCHANGED"),
            ConnectionState::Authed => write!(f, "AUTHED"),
            ConnectionState::InGame => write!(f, "IN_GAME"),
            ConnectionState::Disconnecting => write!(f, "DISCONNECTING"),
        }
    }
}

