<script lang="ts">
    import { useSpacetimeDB, useTable, useReducer } from "spacetimedb/svelte";
    import type { PageProps } from "./$types";
    import { Identity } from "spacetimedb";
    import { reducers, tables } from "$lib/module_bindings";
    import { getUsersMap, getUserUsername } from "$lib/user";
    import { Separator } from "$lib/components/ui/separator";
    import UserCard from "$lib/components/user-card.svelte";
    import { ScrollArea } from "$lib/components/ui/scroll-area";
    import { InputGroup, InputGroupTextarea } from "$lib/components/ui/input-group";
    import { Message, User, type ReceiverIdentity } from "$lib/module_bindings/types";
    import MessageComponent from "$lib/components/message.svelte";
    import GroupCard from "$lib/components/group-card.svelte";
    import { getGroupName } from "$lib/group";

    const conn = useSpacetimeDB();
    const [users] = useTable(tables.user);
    const [groups] = useTable(tables.groups);
    const [messages] = useTable(tables.messages);
    const sendMessage = useReducer(reducers.sendMessage);

    const users_map = $derived(getUsersMap($users as User[]));

    const { params }: PageProps = $props();

    const { user, username, group, groupname, receiver } = $derived.by(() => {
        const split = params.identity.split(":");
        const conversation_type = split[0];
        const user =
            conversation_type == "user"
                ? $users.find((user) => user.identity.isEqual(Identity.fromString(split[1])))
                : undefined;
        const group =
            conversation_type == "group"
                ? $groups.find((group) => group.id == BigInt(split[1]))
                : undefined;

        return {
            user,
            username: user && getUserUsername(user, $conn.identity?.isEqual(user.identity)),
            group,
            groupname: group && getGroupName(group, users_map),
            receiver: (user
                ? { tag: "User", value: user.identity }
                : group
                  ? { tag: "Group", value: group.id }
                  : undefined) as ReceiverIdentity | undefined
        };
    });

    function shouldDisplayMessage(message: Message): boolean {
        if (!receiver || !$conn.identity) return false;

        if (receiver.tag == "User" && message.receiver.tag == "User") {
            return $conn.identity.isEqual(message.sender)
                ? // message from me
                  receiver.value.isEqual(message.receiver.value)
                : // message from others
                  receiver.value.isEqual(message.sender);
        }

        // all group message
        return (
            receiver.tag == "Group" &&
            message.receiver.tag == "Group" &&
            receiver.value == message.receiver.value
        );
    }

    const current_messages = $derived(
        $messages
            .filter(shouldDisplayMessage)
            .sort((a: Message, b: Message) =>
                Number(a.createdAt.toMillis() - b.createdAt.toMillis())
            )
    );
    let new_message = $state("");
</script>

<svelte:head>
    <title>
        {username || groupname}
    </title>
</svelte:head>

<div class="flex h-full min-h-0 flex-col gap-3">
    <div class="flex">
        {#if user}
            <UserCard {user} />
        {/if}
        {#if group}
            <GroupCard {group} />
        {/if}
    </div>
    <Separator />
    <ScrollArea class="h-full min-h-0">
        <div class="flex flex-col items-end gap-3">
            {#each current_messages as message}
                <MessageComponent {message} />
            {/each}
        </div>
    </ScrollArea>
    <InputGroup>
        <InputGroupTextarea
            rows={Math.min(10, new_message.split("\n").length)}
            bind:value={new_message}
            onkeydown={(e) => {
                if (e.key == "Enter" && !e.shiftKey && receiver) {
                    e.preventDefault();
                    sendMessage({ receiver, message: new_message });
                    new_message = "";
                }
            }}
            placeholder="Envoyer un message..."
            class="min-h-0"
        />
    </InputGroup>
</div>
