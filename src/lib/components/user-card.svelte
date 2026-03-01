<script lang="ts">
    import type { User } from "$lib/module_bindings/types";
    import { useReducer, useSpacetimeDB } from "spacetimedb/svelte";
    import { Badge } from "./ui/badge";
    import { Button } from "./ui/button";
    import { HoverCard, HoverCardContent, HoverCardTrigger } from "./ui/hover-card";
    import { Item, ItemContent, ItemMedia, ItemTitle } from "./ui/item";
    import { CalendarDays } from "@lucide/svelte";
    import { Input } from "./ui/input";
    import { getUserUsername } from "$lib/user";
    import { reducers } from "$lib/module_bindings";
    import Avatar from "./avatar.svelte";
    import { convertAvatar } from "$lib/convert";

    const conn = useSpacetimeDB();
    const setUserUsername = useReducer(reducers.setUserUsername);
    const setUserAvatar = useReducer(reducers.setUserAvatar);

    const {
        user,
        variant,
        class: classname
    }: { user: User; variant?: "default" | "icon"; class?: string } = $props();

    const me = $derived($conn.identity?.isEqual(user.identity));
    const username = $derived(getUserUsername(user, me));
    const avatar_url = $derived(
        user.avatar &&
            URL.createObjectURL(
                new Blob([user.avatar as BlobPart], {
                    type: "image/webp"
                })
            )
    );
    let new_username = $state<string>();

    const { status, status_color } = $derived(
        user.status.tag == "Online"
            ? {
                  status: "Disponible",
                  status_color: "bg-green-500"
              }
            : user.status.tag == "OnCall"
              ? {
                    status: "En appel",
                    status_color: "bg-red-500"
                }
              : user.status.tag == "Offline"
                ? {
                      status: "Déconnecter",
                      status_color: "bg-gray-500"
                  }
                : {
                      status: "Inconnue",
                      status_color: "bg-background"
                  }
    );

    function editUsername() {
        if (new_username === undefined) {
            new_username = user.username || "";
        } else {
            new_username = new_username.trim();
            setUserUsername({
                username: new_username.length ? new_username : undefined
            });
            new_username = undefined;
        }
    }

    async function editAvatar(file?: File) {
        const blob = await convertAvatar(file);

        if (!blob) return;

        setUserAvatar({ avatar: await blob?.bytes() });
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
                <Avatar src={avatar_url} alt={username} />
                <div
                    class={["absolute right-0 bottom-0 size-2.5 rounded-full border", status_color]}
                ></div>
            </HoverCardTrigger>
        </ItemMedia>
        {#if variant != "icon"}
            <ItemContent class="w-full min-w-0">
                <ItemTitle
                    class="block w-full cursor-default overflow-hidden text-nowrap text-ellipsis"
                >
                    {username}
                </ItemTitle>
            </ItemContent>
        {/if}
    </Item>
    <HoverCardContent side="bottom" align="start" class="w-96">
        <div class="grid grid-cols-[auto_1fr] items-center">
            <Avatar src={avatar_url} alt={username} onfile={me ? editAvatar : undefined} />
            {#if new_username === undefined}
                <Button
                    variant="ghost"
                    class={[
                        "ml-1 block w-full overflow-hidden px-1.5 text-left text-nowrap text-ellipsis hover:bg-background",
                        me && "cursor-pointer"
                    ]}
                    onclick={() => me && editUsername()}
                >
                    {username}
                </Button>
            {:else}
                <Input
                    class="ml-2.5"
                    placeholder="Nom d'utilisateur"
                    bind:value={new_username}
                    onkeydown={(e) => e.key == "Enter" && editUsername()}
                    onfocusout={editUsername}
                    autofocus
                />
            {/if}
        </div>
        <div class="mt-2.5 flex gap-2.5">
            <Badge class={["font-bold", status_color]}>{status}</Badge>
            {#if user.status.tag == "Offline"}
                <div class="flex items-center text-sm text-muted-foreground">
                    depuis
                    <CalendarDays class="h-4" />
                    {user.status.value.toDate().toLocaleString()}
                </div>
            {/if}
        </div>
    </HoverCardContent>
</HoverCard>
