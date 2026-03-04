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
    import { Call, Message, User, type ReceiverIdentity } from "$lib/module_bindings/types";
    import MessageComponent from "$lib/components/message.svelte";
    import GroupCard from "$lib/components/group-card.svelte";
    import { getGroupName } from "$lib/group";
    import { Button } from "$lib/components/ui/button";
    import { Airplay, Phone, PhoneOff, Video } from "@lucide/svelte";
    import Time from "$lib/components/time.svelte";
    import { compareReceivers } from "$lib";
    import CallCard from "$lib/components/call-card.svelte";

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

    function shouldDisplay(element_sender: Identity, element_receiver: ReceiverIdentity): boolean {
        if (!receiver || !$conn.identity) return false;

        if (receiver.tag == "User" && element_receiver.tag == "User") {
            return $conn.identity.isEqual(element_sender)
                ? // from me
                  receiver.value.isEqual(element_receiver.value)
                : // from others
                  receiver.value.isEqual(element_sender);
        }

        // all group
        return (
            receiver.tag == "Group" &&
            element_receiver.tag == "Group" &&
            receiver.value == element_receiver.value
        );
    }

    const current_messages = $derived(
        $messages
            .filter((message) => shouldDisplay(message.sender, message.receiver))
            .sort((a: Message, b: Message) => Number(b.id - a.id))
    );
    let new_message = $state("");

    const current_calls = $derived(
        $calls
            .filter((call) => shouldDisplay(call.sender, call.receiver))
            .sort((a: Call, b: Call) =>
                Number(b.sender.toString().localeCompare(a.sender.toString()))
            )
    );
    const my_call = $derived($calls.find((call) => $conn.identity?.isEqual(call.sender)));
    const is_my_call = $derived(
        my_call !== undefined &&
            receiver !== undefined &&
            compareReceivers(my_call.receiver, receiver)
    );

    let sharing_camera = $state(false);
    let sharing_screen = $state(false);
</script>

<svelte:head>
    <title>
        {username || groupname}
    </title>
</svelte:head>

<div class="flex min-h-0 min-w-0 flex-col gap-3">
    <div class="flex justify-between gap-3">
        {#if user}
            <UserCard {user} />
        {/if}
        {#if group}
            <GroupCard {group} />
        {/if}
        <div class="flex items-center gap-3">
            {#if is_my_call && my_call}
                <Time timestamp={my_call.startAt} />
            {/if}
            <Button
                variant={is_my_call && sharing_screen ? "default" : "outline"}
                size="icon"
                class="cursor-pointer"
                onclick={async () => {
                    const is_my_call_local = is_my_call;

                    if (is_my_call_local) sharing_screen = !sharing_screen;
                    else sharing_camera = false;

                    if (receiver) await startCall({ receiver });

                    if (!is_my_call_local) sharing_screen = true;
                }}
            >
                <Airplay />
            </Button>
            <Button
                variant={is_my_call && sharing_camera ? "default" : "outline"}
                size="icon"
                class="cursor-pointer"
                onclick={async () => {
                    const is_my_call_local = is_my_call;

                    if (is_my_call_local) sharing_camera = !sharing_camera;
                    else sharing_screen = false;

                    if (receiver) await startCall({ receiver });

                    if (!is_my_call_local) sharing_camera = true;
                }}
            >
                <Video />
            </Button>
            {#if is_my_call}
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
    <div class={["grid h-full min-h-0 gap-3", current_calls.length && "grid-cols-[1fr_auto_18em]"]}>
        {#if current_calls.length}
            <ScrollArea class="min-h-0 min-w-0">
                <div
                    class="flex min-h-[calc(100dvh-6em)] min-w-0 flex-wrap items-center justify-evenly gap-3"
                >
                    {#each current_calls as current_call}
                        {#key current_call.sender.toString()}
                            <CallCard call={current_call} />
                        {/key}
                    {/each}
                </div>
            </ScrollArea>
            <Separator orientation="vertical" />
        {/if}
        <div class="flex min-h-0 flex-col gap-3">
            <ScrollArea class="h-full min-h-0">
                <div class="flex flex-col-reverse items-end gap-3">
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
                            if (new_message.trim().length) {
                                sendMessage({ receiver, message: new_message });
                                new_message = "";
                            }
                        }
                    }}
                    placeholder="Envoyer un message..."
                    class="min-h-0"
                />
            </InputGroup>
        </div>
    </div>
</div>
