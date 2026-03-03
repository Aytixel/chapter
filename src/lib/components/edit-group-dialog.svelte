<script lang="ts">
    import { useReducer, useSpacetimeDB, useTable } from "spacetimedb/svelte";
    import Avatar from "./avatar.svelte";
    import { Checkbox } from "./ui/checkbox";
    import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "./ui/dialog";
    import { Input } from "./ui/input";
    import { Label } from "./ui/label";
    import { ScrollArea } from "./ui/scroll-area";
    import { Separator } from "./ui/separator";
    import UserCard from "./user-card.svelte";
    import { reducers, tables } from "$lib/module_bindings";
    import { Tooltip, TooltipContent, TooltipTrigger } from "./ui/tooltip";
    import { convertAvatar } from "$lib/convert";
    import type { Snippet } from "svelte";
    import { Group, type User } from "$lib/module_bindings/types";
    import { getGroupName } from "$lib/group";
    import { getUsersMap } from "$lib/user";
    import { Crown, Funnel, Trash } from "@lucide/svelte";
    import { Button } from "./ui/button";
    import {
        ContextMenu,
        ContextMenuContent,
        ContextMenuItem,
        ContextMenuTrigger
    } from "./ui/context-menu";

    const {
        group,
        child: trigger_child
    }: { group: Group; child?: Snippet<[{ props: Record<string, unknown> }]> } = $props();

    let user_search = $state("");

    const conn = useSpacetimeDB();
    const [users] = useTable(tables.user);
    const deleteGroup = useReducer(reducers.deleteGroup);
    const addGroupUsers = useReducer(reducers.addGroupUsers);
    const removeGroupUsers = useReducer(reducers.removeGroupUsers);
    const setGroupOwner = useReducer(reducers.setGroupOwner);
    const setGroupName = useReducer(reducers.setGroupName);
    const setGroupAvatar = useReducer(reducers.setGroupAvatar);

    const owner = $derived($conn.identity?.isEqual(group.owner));
    const users_map = $derived(getUsersMap($users as User[]));
    const name = $derived(getGroupName(group, users_map));
    const avatar_url = $derived(
        group.avatar &&
            URL.createObjectURL(
                new Blob([group.avatar as BlobPart], {
                    type: "image/webp"
                })
            )
    );
    let open = $state(false);
    let filter_selected_users = $state(false);
</script>

<Dialog bind:open>
    <DialogTrigger>
        {#snippet child({ props: dialog_props })}
            <Tooltip>
                <TooltipTrigger>
                    {#snippet child({ props: tooltip_props })}
                        {#if trigger_child}
                            {@render trigger_child({
                                props: { ...tooltip_props, ...dialog_props }
                            })}
                        {/if}
                    {/snippet}
                </TooltipTrigger>
                <TooltipContent side="bottom">Voir ou éditer le groupe...</TooltipContent>
            </Tooltip>
        {/snippet}
    </DialogTrigger>
    <DialogContent class="max-h-[min(calc(100dvh-2em),40em)] grid-rows-[repeat(4,auto)_1fr]">
        <DialogHeader>
            <DialogTitle>Voir ou éditer le groupe...</DialogTitle>
        </DialogHeader>
        <div class="flex gap-3">
            <Avatar
                src={avatar_url}
                alt={name.trim().length ? name : "Groupe"}
                variant="square"
                onfile={async (file?: File) => {
                    const blob = await convertAvatar(file);

                    if (!blob) return;

                    setGroupAvatar({
                        groupId: group.id,
                        avatar: await blob?.bytes()
                    });
                }}
            />
            <Input
                type="text"
                placeholder={name}
                value={group.name}
                oninput={(e) =>
                    setGroupName({
                        groupId: group.id,
                        name: e.currentTarget.value
                    })}
            />
            {#if owner}
                <Button
                    variant="destructive"
                    size="icon"
                    onclick={() => {
                        deleteGroup({ groupId: group.id });
                        open = false;
                    }}
                    class="cursor-pointer"
                >
                    <Trash />
                </Button>
            {/if}
        </div>
        <Separator />
        <div class="flex gap-3">
            <Input type="search" placeholder="Rechercher..." bind:value={user_search} />
            <Button
                variant="outline"
                size="icon"
                class="cursor-pointer"
                onclick={() => (filter_selected_users = !filter_selected_users)}
            >
                <Funnel class={[filter_selected_users && "fill-foreground"]} />
            </Button>
        </div>
        <ScrollArea class="h-full  min-h-0">
            <div class="grid gap-1">
                {#each $users.filter((user) => (user.username || user.identity.toString())
                            .toLowerCase()
                            .includes(user_search.trim()) && (!filter_selected_users || group.users.find( (user_identity) => user_identity.isEqual(user.identity) ) !== undefined)) as user}
                    {#key user.identity.toString()}
                        <ContextMenu>
                            <ContextMenuTrigger onpointerdowncapture={(e) => e.stopPropagation()}>
                                <Label
                                    class="flex cursor-pointer justify-between rounded-md pr-3  hover:bg-muted"
                                >
                                    <UserCard {user} class="cursor-pointer" />
                                    {#if user.identity.isEqual(group.owner)}
                                        <Crown class="size-4 stroke-yellow-400" />
                                    {:else}
                                        <Checkbox
                                            checked={group.users.find((user_identity) =>
                                                user_identity.isEqual(user.identity)
                                            ) !== undefined}
                                            onCheckedChange={(checked) =>
                                                (checked ? addGroupUsers : removeGroupUsers)({
                                                    groupId: group.id,
                                                    userIdentities: [user.identity]
                                                })}
                                        />
                                    {/if}
                                </Label>
                            </ContextMenuTrigger>
                            {#if owner && !$conn.identity?.equals(user.identity)}
                                <ContextMenuContent>
                                    <ContextMenuItem
                                        onclick={(e) => {
                                            e.preventDefault();
                                            setGroupOwner({
                                                groupId: group.id,
                                                userIdentity: user.identity
                                            });
                                        }}
                                    >
                                        Transférer la propriété du groupe
                                        <Crown class="size-4 stroke-yellow-400" />
                                    </ContextMenuItem>
                                </ContextMenuContent>
                            {/if}
                        </ContextMenu>
                    {/key}
                {/each}
            </div>
        </ScrollArea>
    </DialogContent>
</Dialog>
