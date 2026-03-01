<script lang="ts">
    import { useSpacetimeDB, useTable } from "spacetimedb/svelte";
    import { Input } from "./ui/input";
    import { ScrollArea } from "./ui/scroll-area";
    import { Separator } from "./ui/separator";
    import UserCard from "./user-card.svelte";
    import { tables } from "$lib/module_bindings";

    const conn = useSpacetimeDB();
    const [users] = useTable(tables.user);
    const me = $derived($users.find((user) => $conn.identity?.isEqual(user.identity)));

    let user_search = $state("");
</script>

<div class="grid h-full min-h-0 grid-rows-[auto_1fr] gap-3">
    <Input placeholder="Conversations..." bind:value={user_search} />
    <ScrollArea class="h-full min-h-0">
        <div class="grid w-72 gap-3">
            {#if me}
                <a href="/conversation/user:{me.identity.toString()}"
                    ><UserCard user={me} class="hover:bg-muted" /></a
                >
            {/if}
            <Separator />
            {#each $users as user}
                {#if (user.username || user.identity.toString())
                    .toLowerCase()
                    .includes(user_search) && !$conn.identity?.isEqual(user.identity)}
                    <a href="/conversation/user:{user.identity.toString()}">
                        <UserCard {user} class="hover:bg-muted" />
                    </a>
                {/if}
            {/each}
        </div>
    </ScrollArea>
</div>
