<script lang="ts">
    import { useReducer, useTable } from "spacetimedb/svelte";
    import Avatar from "./avatar.svelte";
    import { Checkbox } from "./ui/checkbox";
    import {
        Dialog,
        DialogContent,
        DialogDescription,
        DialogHeader,
        DialogTitle,
        DialogTrigger
    } from "./ui/dialog";
    import { Input } from "./ui/input";
    import { Label } from "./ui/label";
    import { ScrollArea } from "./ui/scroll-area";
    import { Separator } from "./ui/separator";
    import UserCard from "./user-card.svelte";
    import { reducers, tables } from "$lib/module_bindings";
    import { Tooltip, TooltipContent, TooltipTrigger } from "./ui/tooltip";
    import { Button } from "./ui/button";
    import { Plus } from "@lucide/svelte";
    import { Identity } from "spacetimedb";
    import { convertAvatar } from "$lib/convert";
    import { SvelteSet } from "svelte/reactivity";
    import type { Snippet } from "svelte";
    import { useSpacetimeDB } from "spacetimedb/svelte";

    const { child: trigger_child }: { child: Snippet<[{ props: Record<string, unknown> }]> } =
        $props();

    const conn = useSpacetimeDB();
    const [users] = useTable(tables.user);
    const createGroup = useReducer(reducers.createGroup);

    let open = $state(false);
    let name = $state("");
    let user_search = $state("");
    let selected_users = new SvelteSet<string>();
    let avatar_url = $state("");
    let avatar: Uint8Array | undefined = undefined;

    $effect(() => {
        if (!open) {
            name = "";
            user_search = "";
            selected_users.clear();
            avatar_url = "";
            avatar = undefined;
        }
    });
</script>

<Dialog bind:open>
    <DialogTrigger>
        {#snippet child({ props: dialog_props })}
            <Tooltip>
                <TooltipTrigger>
                    {#snippet child({ props: tooltip_props })}
                        {@render trigger_child({
                            props: { ...tooltip_props, ...dialog_props }
                        })}
                    {/snippet}
                </TooltipTrigger>
                <TooltipContent side="bottom">Créer un groupe...</TooltipContent>
            </Tooltip>
        {/snippet}
    </DialogTrigger>
    <DialogContent class="max-h-[min(calc(100dvh-2em),40em)] grid-rows-[repeat(4,auto)_1fr]">
        <DialogHeader>
            <DialogTitle>Créer un groupe...</DialogTitle>
            <DialogDescription>
                Converser avec de multiple personnes instantanément.
            </DialogDescription>
        </DialogHeader>
        <div class="flex gap-3">
            <Avatar
                src={avatar_url}
                alt={name.trim().length ? name : "Groupe"}
                variant="square"
                onfile={async (file?: File) => {
                    const blob = await convertAvatar(file);

                    if (!blob) return;

                    avatar = await blob.bytes();
                    avatar_url = URL.createObjectURL(blob);
                }}
            />
            <Input type="text" placeholder="Nom du groupe.." bind:value={name} />
            <Button
                type="submit"
                class="cursor-pointer"
                onclick={async () => {
                    createGroup({
                        name,
                        avatar,
                        userIdentities: [...selected_users].map((identity) =>
                            Identity.fromString(identity)
                        )
                    });
                    open = false;
                }}
            >
                Créer <Plus />
            </Button>
        </div>
        <Separator />
        <Input type="search" placeholder="Rechercher..." bind:value={user_search} />
        <ScrollArea class="h-full  min-h-0">
            <div class="grid gap-1">
                {#each $users.filter((user) => (user.username || user.identity.toString())
                            .toLowerCase()
                            .includes(user_search.trim()) && !$conn.identity?.isEqual(user.identity)) as user}
                    {#key user.identity.toString()}
                        <Label
                            class="flex cursor-pointer justify-between rounded-md pr-3 hover:bg-muted"
                        >
                            <UserCard {user} class="cursor-pointer" />
                            <Checkbox
                                checked={selected_users.has(user.identity.toString())}
                                onCheckedChange={(checked) =>
                                    checked
                                        ? selected_users.add(user.identity.toString())
                                        : selected_users.delete(user.identity.toString())}
                            />
                        </Label>
                    {/key}
                {/each}
            </div>
        </ScrollArea>
    </DialogContent>
</Dialog>
