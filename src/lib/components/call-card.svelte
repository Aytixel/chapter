<script lang="ts">
    import { tables } from "$lib/module_bindings";
    import type { Call } from "$lib/module_bindings/types";
    import { useTable } from "spacetimedb/svelte";
    import Avatar from "./avatar.svelte";
    import { getUserUsername } from "$lib/user";

    const { call }: { call: Call } = $props();

    const [user] = useTable(tables.user.where((user) => user.identity.eq(call.sender)));

    const username = $derived(getUserUsername($user[0]));
    const avatar_url = $derived(
        $user[0].avatar &&
            URL.createObjectURL(
                new Blob([$user[0].avatar as BlobPart], {
                    type: "image/webp"
                })
            )
    );
</script>

{#if $user[0]}
    <div
        class="flex aspect-video h-48 min-h-0 min-w-0 items-center justify-center rounded-md border bg-muted/15"
    >
        <Avatar src={avatar_url} alt={username} class="size-32 text-5xl" />
    </div>
{/if}
