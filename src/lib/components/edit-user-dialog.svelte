<script lang="ts">
    import type { Snippet } from "svelte";
    import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "./ui/dialog";
    import { Tooltip, TooltipContent, TooltipTrigger } from "./ui/tooltip";
    import type { User } from "$lib/module_bindings/types";
    import { useReducer, useSpacetimeDB } from "spacetimedb/svelte";
    import { getUserStatus, getUserUsername } from "$lib/user";
    import { reducers } from "$lib/module_bindings";
    import { convertAvatar } from "$lib/convert";
    import { Input } from "./ui/input";
    import Avatar from "./avatar.svelte";
    import { Badge } from "./ui/badge";
    import { CalendarDays } from "@lucide/svelte";
    import { Separator } from "./ui/separator";

    const {
        user,
        child: trigger_child
    }: { user: User; child?: Snippet<[{ props: Record<string, unknown> }]> } = $props();

    const conn = useSpacetimeDB();
    const setUserUsername = useReducer(reducers.setUserUsername);
    const setUserAvatar = useReducer(reducers.setUserAvatar);

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

    const { status, status_color } = $derived(getUserStatus(user.status));
</script>

<Dialog>
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
                <TooltipContent side="bottom"
                    >Voir {me ? "ou éditer" : ""} le profil...</TooltipContent
                >
            </Tooltip>
        {/snippet}
    </DialogTrigger>
    <DialogContent class="max-h-[min(calc(100dvh-2em),40em)] grid-rows-[repeat(4,auto)_1fr]">
        <DialogHeader>
            <DialogTitle>Voir {me ? "ou éditer" : ""} le profil...</DialogTitle>
        </DialogHeader>
        <div class="grid grid-cols-[auto_1fr] items-center gap-3">
            <Avatar
                src={avatar_url}
                alt={username}
                onfile={me
                    ? async (file?: File) => {
                          const blob = await convertAvatar(file);

                          if (!blob) return;

                          setUserAvatar({
                              avatar: await blob?.bytes()
                          });
                      }
                    : undefined}
            />
            {#if me}
                <Input
                    type="text"
                    placeholder={username}
                    value={user.username}
                    oninput={(e) =>
                        setUserUsername({
                            username: e.currentTarget.value
                        })}
                />
            {:else}
                <div class="overflow-hidden text-ellipsis">{username}</div>
            {/if}
        </div>
        <Separator />
        <div class="flex justify-between gap-3">
            <Badge class={["font-bold", status_color]}>{status}</Badge>
            {#if user.status.tag == "Offline"}
                <div class="flex items-center text-sm text-muted-foreground">
                    depuis
                    {user.status.value.toDate().toLocaleString()}
                    <CalendarDays class="h-4" />
                </div>
            {/if}
        </div>
    </DialogContent>
</Dialog>
