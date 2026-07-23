use super::super::ConnectionId;

/// One session-scoped field group cleared on Connect Command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreshSessionReset {
    Session,
    Stats,
    SecondaryStatus,
    CombatAwareness,
    TelnetBuffer,
    GuildSelection,
    Automation,
    UserConfigLoaded,
    PlayerProfile,
    GenericCommands,
    Dialogs,
}

impl FreshSessionReset {
    pub const ALL: [Self; 11] = [
        Self::Session,
        Self::Stats,
        Self::SecondaryStatus,
        Self::CombatAwareness,
        Self::TelnetBuffer,
        Self::GuildSelection,
        Self::Automation,
        Self::UserConfigLoaded,
        Self::PlayerProfile,
        Self::GenericCommands,
        Self::Dialogs,
    ];
}

/// Authoritative manifest of session-scoped runtime state cleared on `/connect`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreshSessionPlan {
    pub connection_id: ConnectionId,
}

impl FreshSessionPlan {
    pub(crate) fn new(connection_id: ConnectionId) -> Self {
        Self { connection_id }
    }

    pub fn resets(&self) -> &'static [FreshSessionReset] {
        &FreshSessionReset::ALL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_covers_every_reset_kind() {
        assert_eq!(
            FreshSessionPlan::new(1).resets().len(),
            FreshSessionReset::ALL.len()
        );
    }
}
