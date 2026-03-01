<script lang="ts">
    import type { Group, User } from "$lib/module_bindings/types";
    import { useReducer, useSpacetimeDB, useTable } from "spacetimedb/svelte";
    import Avatar from "./avatar.svelte";
    import { Button } from "./ui/button";
    import { HoverCard, HoverCardContent, HoverCardTrigger } from "./ui/hover-card";
    import { Input } from "./ui/input";
    import { Item, ItemContent, ItemMedia, ItemTitle } from "./ui/item";
    import { reducers, tables } from "$lib/module_bindings";
    import { convertAvatar } from "$lib/convert";
    import { getGroupName } from "$lib/group";
    import UserCard from "./user-card.svelte";
    import { getUsersMap } from "$lib/user";
    import { Separator } from "./ui/separator";

    const conn = useSpacetimeDB();
    const [users] = useTable(tables.user);
    const setGroupName = useReducer(reducers.setGroupName);
    const setGroupAvatar = useReducer(reducers.setGroupAvatar);

    const {
        group,
        variant,
        class: classname
    }: { group: Group; variant?: "default" | "icon"; class?: string } = $props();

    const owner = $derived($conn.identity?.isEqual(group.owner));
    const users_map = $derived(getUsersMap($users as User[]));
    const avatar_url = $derived(
        group.avatar &&
            URL.createObjectURL(
                new Blob([group.avatar as BlobPart], {
                    type: "image/webp"
                })
            )
    );
    const name = $derived(getGroupName(group, users_map));
    let new_name = $state<string>();

    function editName() {
        if (new_name === undefined) {
            new_name = group.name || "";
        } else {
            new_name = new_name.trim();
            setGroupName({
                groupId: group.id,
                name: new_name.length ? new_name : undefined
            });
            new_name = undefined;
        }
    }

    async function editAvatar(file?: File) {
        const blob = await convertAvatar(file);

        if (!blob) return;

        setGroupAvatar({
            groupId: group.id,
            avatar: await blob?.bytes()
        });
    }
</script>

<HoverCard>
    <Item
        class={[
            variant == "icon" ? "w-fit p-0" : "grid grid-cols-[auto_1fr] p-1 transition-colors",
            classname
        ]}
        size="sm"
    >
        <ItemMedia class="relative translate-0!">
            <HoverCardTrigger class="cursor-pointer">
                <Avatar src={avatar_url} alt={name} variant="square" />
            </HoverCardTrigger>
        </ItemMedia>
        {#if variant != "icon"}
            <ItemContent class="w-full min-w-0">
                <ItemTitle
                    class="block w-full cursor-default overflow-hidden text-nowrap text-ellipsis"
                >
                    {name}
                </ItemTitle>
            </ItemContent>
        {/if}
    </Item>
    <HoverCardContent side="bottom" align="start" class="grid w-96 gap-3">
        <div class="grid grid-cols-[auto_1fr] items-center">
            <Avatar src={avatar_url} alt={name} variant="square" onfile={editAvatar} />
            {#if new_name === undefined}
                <Button
                    variant="ghost"
                    class="hover:bg-backgroundcursor-pointer ml-1 block w-full cursor-pointer overflow-hidden px-1.5 text-left text-nowrap text-ellipsis"
                    onclick={() => editName()}
                >
                    {name}
                </Button>
            {:else}
                <Input
                    class="ml-2.5"
                    placeholder="Nom d'utilisateur"
                    bind:value={new_name}
                    onkeydown={(e) => e.key == "Enter" && editName()}
                    onfocusout={editName}
                    autofocus
                />
            {/if}
        </div>
        <Separator />
        <div class="grid gap-1">
            {#each group.users as user_identity}
                {@const user = users_map.get(user_identity.toString())}
                {#if user}
                    <UserCard {user} class="pl-0" />
                {/if}
            {/each}
        </div>
    </HoverCardContent>
</HoverCard>
