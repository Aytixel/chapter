<script lang="ts">
    import { useSpacetimeDB } from "spacetimedb/svelte";
    import { Separator } from "$lib/components/ui/separator";
    import ConversationList from "$lib/components/conversation-list.svelte";

    const conn = useSpacetimeDB();
</script>

<svelte:head>
    <title>SpacetimeDB Svelte App</title>
</svelte:head>

{#if $conn.isActive}
    <div class="grid h-dvh grid-cols-[auto_auto_1fr] gap-3 p-3">
        <ConversationList />
        <Separator orientation="vertical" />
        <div class="h-full"></div>
    </div>
{/if}
