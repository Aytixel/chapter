use std::collections::HashSet;

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
    name: Option<String>,
    avatar: Option<Vec<u8>>,
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
    let messages_from_group = groups(ctx)
        .into_iter()
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
    if let Some(user) = get_user(ctx, ctx.sender()) {
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
    if let Some(user) = get_user(ctx, ctx.sender()) {
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
pub fn set_user_username(ctx: &ReducerContext, username: Option<String>) -> Result<(), String> {
    let Some(user) = get_user(ctx, ctx.sender()) else {
        return Err("Cannot set username for unknown user".to_string());
    };

    ctx.db.user().identity().update(User {
        username: username
            .map(|username| username.trim().to_string())
            .filter(|username| !username.is_empty()),
        ..user
    });

    Ok(())
}

#[spacetimedb::reducer]
pub fn set_user_avatar(ctx: &ReducerContext, avatar: Option<Vec<u8>>) -> Result<(), String> {
    if let Some(avatar) = &avatar
        && avatar.len() > 1_000_000
    {
        return Err("Avatar picture too large, the limit 1MB.".to_string());
    }

    let Some(user) = get_user(ctx, ctx.sender()) else {
        return Err("Cannot set avatar for unknown user".to_string());
    };

    ctx.db.user().identity().update(User { avatar, ..user });

    Ok(())
}

fn get_user(ctx: &ReducerContext, user_identity: Identity) -> Option<User> {
    ctx.db.user().identity().find(user_identity)
}

#[spacetimedb::reducer]
pub fn create_group(
    ctx: &ReducerContext,
    name: Option<String>,
    avatar: Option<Vec<u8>>,
    user_identities: Vec<Identity>,
) -> Result<(), String> {
    let Some(user) = get_user(ctx, ctx.sender()) else {
        return Err("Cannot create group with unknown user".to_string());
    };

    let mut groups: HashSet<u256> = HashSet::from_iter(user.groups);

    let group = ctx.db.group().insert(Group {
        id: u256::new(0),
        owner: ctx.sender(),
        name: None,
        avatar: None,
        users: vec![user.identity],
    });

    groups.insert(group.id);

    ctx.db.user().identity().update(User {
        groups: groups.into_iter().collect(),
        ..user
    });

    set_group_name(ctx, group.id, name)?;
    set_group_avatar(ctx, group.id, avatar)?;
    add_group_users(ctx, group.id, user_identities)?;

    Ok(())
}

#[spacetimedb::reducer]
pub fn delete_group(ctx: &ReducerContext, group_id: u256) -> Result<(), String> {
    let Some(group) = get_group(ctx, group_id) else {
        return Err("Cannot delete unknown group".to_string());
    };

    if get_user(ctx, group.owner).is_none() {
        return Err("Cannot delete group if your are not owner".to_string());
    }

    for user_identity in group.users {
        let Some(user) = get_user(ctx, user_identity) else {
            continue;
        };

        let mut groups: HashSet<u256> = HashSet::from_iter(user.groups);

        groups.remove(&group_id);

        ctx.db.user().identity().update(User {
            groups: groups.into_iter().collect(),
            ..user
        });
    }

    ctx.db.group().id().delete(group_id);

    Ok(())
}

#[spacetimedb::reducer]
pub fn add_group_users(
    ctx: &ReducerContext,
    group_id: u256,
    user_identities: Vec<Identity>,
) -> Result<(), String> {
    if user_identities.len() > 1000 {
        return Err("Cannot add more than 1000 users simultaneously".to_string());
    }

    let Some(group) = get_group(ctx, group_id) else {
        return Err("Cannot add users for unknown group".to_string());
    };

    let mut users: HashSet<Identity> = HashSet::from_iter(group.users);

    for user_identity in user_identities {
        let Some(user) = get_user(ctx, user_identity) else {
            continue;
        };
        let mut groups: HashSet<u256> = HashSet::from_iter(user.groups);

        users.insert(user_identity);
        groups.insert(group_id);

        ctx.db.user().identity().update(User {
            groups: groups.into_iter().collect(),
            ..user
        });
    }

    ctx.db.group().id().update(Group {
        users: users.into_iter().collect(),
        ..group
    });

    Ok(())
}

#[spacetimedb::reducer]
pub fn remove_group_users(
    ctx: &ReducerContext,
    group_id: u256,
    user_identities: Vec<Identity>,
) -> Result<(), String> {
    if user_identities.len() > 1000 {
        return Err("Cannot remove more than 1000 users simultaneously".to_string());
    }

    let Some(group) = get_group(ctx, group_id) else {
        return Err("Cannot remove users for unknown group".to_string());
    };

    let mut users: HashSet<Identity> = HashSet::from_iter(group.users);

    for user_identity in user_identities {
        let Some(user) = get_user(ctx, user_identity) else {
            continue;
        };

        if user.identity == group.owner {
            continue;
        }

        let mut groups: HashSet<u256> = HashSet::from_iter(user.groups);

        users.remove(&user_identity);
        groups.remove(&group_id);

        ctx.db.user().identity().update(User {
            groups: groups.into_iter().collect(),
            ..user
        });
    }

    ctx.db.group().id().update(Group {
        users: users.into_iter().collect(),
        ..group
    });

    Ok(())
}

#[spacetimedb::reducer]
pub fn set_group_owner(
    ctx: &ReducerContext,
    group_id: u256,
    user_identity: Identity,
) -> Result<(), String> {
    let Some(group) = get_group(ctx, group_id) else {
        return Err("Cannot set owner for unknown group".to_string());
    };

    if group.owner == ctx.sender() {
        return Err("Only group owner can set group owner".to_string());
    }

    if get_user(ctx, user_identity).is_none() {
        return Err("Cannot set unknown user as group owner".to_string());
    }

    ctx.db.group().id().update(Group {
        owner: user_identity,
        ..group
    });

    Ok(())
}

#[spacetimedb::reducer]
pub fn set_group_name(
    ctx: &ReducerContext,
    group_id: u256,
    name: Option<String>,
) -> Result<(), String> {
    let Some(group) = get_group(ctx, group_id) else {
        return Err("Cannot set name for unknown group".to_string());
    };

    ctx.db.group().id().update(Group {
        name: name
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty()),
        ..group
    });

    Ok(())
}

#[spacetimedb::reducer]
pub fn set_group_avatar(
    ctx: &ReducerContext,
    group_id: u256,
    avatar: Option<Vec<u8>>,
) -> Result<(), String> {
    if let Some(avatar) = &avatar
        && avatar.len() > 1_000_000
    {
        return Err("Avatar picture too large, the limit 1MB.".to_string());
    }

    let Some(group) = get_group(ctx, group_id) else {
        return Err("Cannot set avatar for unknown group".to_string());
    };

    ctx.db.group().id().update(Group { avatar, ..group });

    Ok(())
}

fn get_group(ctx: &ReducerContext, group_id: u256) -> Option<Group> {
    ctx.db.group().id().find(group_id)
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

fn check_message(message: &str) -> Result<(), String> {
    if message.len() > u16::MAX as usize {
        return Err(format!(
            "Message too large, the limit {} character.",
            u16::MAX
        ));
    }

    Ok(())
}
