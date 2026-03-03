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
    import { Button } from "$lib/components/ui/button";
    import { Airplay, Phone, PhoneOff, Video } from "@lucide/svelte";
    import Time from "$lib/components/time.svelte";

    const { params }: PageProps = $props();

    const conn = useSpacetimeDB();
    const [users] = useTable(tables.user);
    const [groups] = useTable(tables.groups);
    const [messages] = useTable(tables.messages);
    const [calls] = useTable(tables.calls);
    const sendMessage = useReducer(reducers.sendMessage);
    const startCall = useReducer(reducers.startCall);
    const stopCall = useReducer(reducers.stopCall);

    const users_map = $derived(getUsersMap($users as User[]));

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

    $effect(() => {
        if (!receiver) location.href = "/";
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
        $messages.filter(shouldDisplayMessage).sort((a: Message, b: Message) => Number(a.id - b.id))
    );
    let new_message = $state("");

    const current_call = $derived($calls.find((call) => $conn.identity?.isEqual(call.sender)));
    const is_current_call = $derived(
        current_call !== undefined &&
            receiver !== undefined &&
            (current_call.receiver.tag == "User" && receiver.tag == "User"
                ? current_call.receiver.value.isEqual(receiver.value)
                : current_call.receiver.tag == "Group" && receiver.tag == "Group"
                  ? current_call.receiver.value == receiver.value
                  : false)
    );

    let sharing_camera = $state(false);
    let sharing_screen = $state(false);
</script>

<svelte:head>
    <title>
        {username || groupname}
    </title>
</svelte:head>

<div class="flex h-full min-h-0 flex-col gap-3">
    <div class="flex justify-between gap-3">
        {#if user}
            <UserCard {user} />
        {/if}
        {#if group}
            <GroupCard {group} />
        {/if}
        <div class="flex items-center gap-3">
            {#if is_current_call && current_call}
                <Time timestamp={current_call.startAt} />
            {/if}
            <Button
                variant={is_current_call && sharing_screen ? "default" : "outline"}
                size="icon"
                class="cursor-pointer"
                onclick={async () => {
                    const is_current_call_local = is_current_call;

                    if (is_current_call_local) sharing_screen = !sharing_screen;
                    else sharing_camera = false;

                    if (receiver) await startCall({ receiver });

                    if (!is_current_call_local) sharing_screen = true;
                }}
            >
                <Airplay />
            </Button>
            <Button
                variant={is_current_call && sharing_camera ? "default" : "outline"}
                size="icon"
                class="cursor-pointer"
                onclick={async () => {
                    const is_current_call_local = is_current_call;

                    if (is_current_call_local) sharing_camera = !sharing_camera;
                    else sharing_screen = false;

                    if (receiver) await startCall({ receiver });

                    if (!is_current_call_local) sharing_camera = true;
                }}
            >
                <Video />
            </Button>
            {#if is_current_call}
                <Button
                    variant="destructive"
                    size="icon"
                    class="cursor-pointer"
                    onclick={() => {
                        sharing_camera = false;
                        sharing_screen = false;

                        stopCall();
                    }}
                >
                    <PhoneOff />
                </Button>
            {:else}
                <Button
                    variant="outline"
                    size="icon"
                    class="cursor-pointer"
                    onclick={() => {
                        sharing_camera = false;
                        sharing_screen = false;

                        if (receiver) startCall({ receiver });
                    }}
                >
                    <Phone />
                </Button>
            {/if}
        </div>
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
