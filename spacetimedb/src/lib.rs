use spacetimedb::{Identity, ReducerContext, SpacetimeType, Table, Timestamp};

#[spacetimedb::table(accessor = user, public)]
pub struct User {
    #[primary_key]
    identity: Identity,
    status: UserStatus,
    username: Option<String>,
    avatar: Option<Vec<u8>>,
}

#[derive(SpacetimeType)]
pub enum UserStatus {
    Online,
    Offline { at: Timestamp },
    OnCall,
}

#[spacetimedb::reducer(init)]
pub fn init(_ctx: &ReducerContext) {
    // Called when the module is initially published
}

#[spacetimedb::reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender()) {
        ctx.db.user().identity().update(User {
            status: UserStatus::Online,
            ..user
        });
    } else {
        ctx.db.user().insert(User {
            identity: ctx.sender(),
            status: UserStatus::Online,
            username: None,
            avatar: None,
        });
    }
}

#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender()) {
        ctx.db.user().identity().update(User {
            status: UserStatus::Offline { at: ctx.timestamp },
            ..user
        });
    } else {
        log::warn!(
            "Disconnect event for unknown user with identity {:?}",
            ctx.sender()
        );
    }
}

#[spacetimedb::reducer]
pub fn set_username(ctx: &ReducerContext, username: Option<String>) -> Result<(), String> {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender()) {
        ctx.db.user().identity().update(User { username, ..user });
        Ok(())
    } else {
        Err("Cannot set username for unknown user".to_string())
    }
}

#[spacetimedb::reducer]
pub fn set_avatar(ctx: &ReducerContext, avatar: Option<Vec<u8>>) -> Result<(), String> {
    if let Some(avatar) = &avatar
        && avatar.len() > 1_000_000
    {
        return Err("Avatar picture is too large, the limit 1MB.".to_string());
    }

    if let Some(user) = ctx.db.user().identity().find(ctx.sender()) {
        ctx.db.user().identity().update(User { avatar, ..user });
        Ok(())
    } else {
        Err("Cannot set avatar for unknown user".to_string())
    }
}
