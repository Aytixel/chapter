use spacetimedb::{
    FilterableValue, Identity, ReducerContext, SpacetimeType, Table, Timestamp, ViewContext,
    sats::u256, spacetimedb_lib::Private,
};

#[spacetimedb::table(accessor = user, public)]
pub struct User {
    #[primary_key]
    identity: Identity,
    status: UserStatus,
    username: Option<String>,
    avatar: Option<Vec<u8>>,
    groups: Vec<u256>,
}

#[derive(SpacetimeType)]
pub enum UserStatus {
    Online,
    Offline { at: Timestamp },
    OnCall,
}

#[spacetimedb::table(accessor = group)]
pub struct Group {
    #[primary_key]
    #[auto_inc]
    id: u256,
    owner: Identity,
    name: String,
    users: Vec<Identity>,
}

#[derive(SpacetimeType)]
pub enum ReceiverIdentity {
    User { identity: Identity },
    Group { id: u256 },
}

impl Private for ReceiverIdentity {}

impl FilterableValue for ReceiverIdentity {
    type Column = ReceiverIdentity;
}

#[spacetimedb::view(accessor = groups, public)]
fn groups(ctx: &ViewContext) -> Vec<Group> {
    let Some(user) = ctx.db.user().identity().find(ctx.sender()) else {
        return Vec::new();
    };

    user.groups
        .iter()
        .flat_map(|group| ctx.db.group().id().find(group))
        .collect()
}

#[spacetimedb::table(accessor = message)]
pub struct Message {
    #[primary_key]
    #[auto_inc]
    id: u256,
    #[index(btree)]
    receiver: ReceiverIdentity,
    #[index(btree)]
    sender: Identity,
    message: String,
    created_at: Timestamp,
    updated_at: Timestamp,
}

#[spacetimedb::view(accessor = messages, public)]
fn messages(ctx: &ViewContext) -> Vec<Message> {
    let identity = ctx.sender();
    let Some(user) = ctx.db.user().identity().find(&identity) else {
        return Vec::new();
    };

    let messages_from_group = user
        .groups
        .iter()
        .flat_map(|group| ctx.db.group().id().find(group))
        .flat_map(|group| {
            ctx.db
                .message()
                .receiver()
                .filter(ReceiverIdentity::Group { id: group.id })
        })
        .filter(|message| message.sender != identity);
    let messages_from_user = ctx
        .db
        .message()
        .receiver()
        .filter(ReceiverIdentity::User { identity });
    let message_from_sender = ctx.db.message().sender().filter(&identity);

    messages_from_group
        .chain(messages_from_user)
        .chain(message_from_sender)
        .collect()
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
            groups: Vec::new(),
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
    let Some(user) = ctx.db.user().identity().find(ctx.sender()) else {
        return Err("Cannot set username for unknown user".to_string());
    };

    ctx.db.user().identity().update(User { username, ..user });

    Ok(())
}

#[spacetimedb::reducer]
pub fn set_avatar(ctx: &ReducerContext, avatar: Option<Vec<u8>>) -> Result<(), String> {
    if let Some(avatar) = &avatar
        && avatar.len() > 1_000_000
    {
        return Err("Avatar picture too large, the limit 1MB.".to_string());
    }

    let Some(user) = ctx.db.user().identity().find(ctx.sender()) else {
        return Err("Cannot set avatar for unknown user".to_string());
    };

    ctx.db.user().identity().update(User { avatar, ..user });

    Ok(())
}

fn check_message(message: &str) -> Result<(), String> {
    if message.len() > u16::MAX as usize {
        return Err(format!(
            "Message too large, the limit {} character.",
            u16::MAX
        ));
    }

    Ok(())
}

#[spacetimedb::reducer]
pub fn send_message(
    ctx: &ReducerContext,
    receiver: ReceiverIdentity,
    message: String,
) -> Result<(), String> {
    check_message(&message)?;

    match receiver {
        ReceiverIdentity::User { identity }
            if ctx.db.user().identity().find(identity).is_none() =>
        {
            return Err("Cannot send message to unknown user".to_string());
        }
        ReceiverIdentity::Group { id } if ctx.db.group().id().find(id).is_none() => {
            return Err("Cannot send message to unknown group".to_string());
        }
        _ => {}
    }

    ctx.db.message().insert(Message {
        id: u256::new(0),
        receiver,
        sender: ctx.sender(),
        message,
        created_at: ctx.timestamp,
        updated_at: ctx.timestamp,
    });

    Ok(())
}

#[spacetimedb::reducer]
pub fn update_message(
    ctx: &ReducerContext,
    message_id: u256,
    message: String,
) -> Result<(), String> {
    check_message(&message)?;

    let Some(old_message) = ctx.db.message().id().find(message_id) else {
        return Err("Cannot update unknown message".to_string());
    };

    if old_message.sender != ctx.sender() {
        return Err("Cannot update message sent by another".to_string());
    }

    ctx.db.message().id().update(Message {
        message,
        updated_at: ctx.timestamp,
        ..old_message
    });

    Ok(())
}

#[spacetimedb::reducer]
pub fn delete_message(ctx: &ReducerContext, message_id: u256) -> Result<(), String> {
    let Some(old_message) = ctx.db.message().id().find(message_id) else {
        return Err("Cannot update unknown message".to_string());
    };

    if old_message.sender != ctx.sender() {
        return Err("Cannot update message sent by another".to_string());
    }

    ctx.db.message().delete(old_message);

    Ok(())
}
