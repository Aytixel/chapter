<script lang="ts">
    import { useSpacetimeDB, useTable } from "spacetimedb/svelte";
    import { Input } from "./ui/input";
    import { ScrollArea } from "./ui/scroll-area";
    import { Separator } from "./ui/separator";
    import UserCard from "./user-card.svelte";
    import { tables } from "$lib/module_bindings";
    import CreateGroup from "./consersation-list/create-group.svelte";
    import GroupCard from "./group-card.svelte";
    import { getUsersMap, getUserUsername } from "$lib/user";
    import { getGroupName } from "$lib/group";
    import { Button } from "./ui/button";
    import type { User } from "$lib/module_bindings/types";
    import { Moon, Sun } from "@lucide/svelte";

    const conn = useSpacetimeDB();
    const [users] = useTable(tables.user);
    const [groups] = useTable(tables.groups);

    const me = $derived($users.find((user) => $conn.identity?.isEqual(user.identity)));
    const users_map = $derived(getUsersMap($users as User[]));

    let search = $state("");
    const filtered_groups = $derived(
        $groups.filter((group) => getGroupName(group, users_map).toLowerCase().includes(search))
    );
    const filtered_users = $derived(
        $users.filter(
            (user) =>
                getUserUsername(user).toLowerCase().includes(search) &&
                !$conn.identity?.isEqual(user.identity)
        )
    );

    let dark_theme = $state(localStorage.getItem("dark_theme") !== null);

    $effect(() => {
        document.body.classList.toggle("dark", dark_theme);

        if (dark_theme) localStorage.setItem("dark_theme", "");
        else localStorage.removeItem("dark_theme");
    });
</script>

<div class="grid h-full min-h-0 grid-rows-[auto_1fr] gap-3">
    <div class="flex gap-3">
        <Input type="search" placeholder="Rechercher..." bind:value={search} />
        <CreateGroup />
    </div>
    <ScrollArea class="h-full min-h-0">
        <div class="grid w-72 gap-1">
            {#if me}
                <a href="/conversation/user:{me.identity.toString()}">
                    <UserCard user={me} class="hover:bg-muted" />
                </a>
            {/if}
            {#if filtered_groups.length}
                <Separator class="my-2" />
                {#each filtered_groups as group}
                    <a href="/conversation/group:{group.id}">
                        <GroupCard {group} class="hover:bg-muted" />
                    </a>
                {/each}
            {/if}
            <Separator class="my-2" />
            {#each filtered_users as user}
                <a href="/conversation/user:{user.identity.toString()}">
                    <UserCard {user} class="hover:bg-muted" />
                </a>
            {/each}
        </div>
    </ScrollArea>
    <div>
        <Button size="icon" variant="outline" onclick={() => (dark_theme = !dark_theme)}>
            {#if dark_theme}
                <Moon />
            {:else}
                <Sun />
            {/if}
        </Button>
    </div>
</div>
