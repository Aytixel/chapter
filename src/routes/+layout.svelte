<script lang="ts">
    import "./layout.css";
    import favicon from "$lib/assets/favicon.svg";
    import { createSpacetimeDBProvider, useSpacetimeDB } from "spacetimedb/svelte";
    import type { Identity } from "spacetimedb";
    import { DbConnection, type ErrorContext } from "$lib/module_bindings";
    import { Separator } from "$lib/components/ui/separator";
    import ConversationList from "$lib/components/conversation-list.svelte";

    const HOST = import.meta.env.VITE_SPACETIMEDB_HOST ?? "ws://localhost:3000";
    const DB_NAME = import.meta.env.VITE_SPACETIMEDB_DB_NAME ?? "chapter";
    const TOKEN_KEY = `${HOST}/${DB_NAME}/auth_token`;

    const onConnect = (_conn: DbConnection, identity: Identity, token: string) => {
        localStorage.setItem(TOKEN_KEY, token);
        console.log("Connected to SpacetimeDB with identity:", identity.toHexString());
    };

    const onDisconnect = () => {
        console.log("Disconnected from SpacetimeDB");
    };

    const onConnectError = (_ctx: ErrorContext, err: Error) => {
        console.log("Error connecting to SpacetimeDB:", err);
    };

    const connectionBuilder = DbConnection.builder()
        .withUri(HOST)
        .withDatabaseName(DB_NAME)
        .withToken(localStorage.getItem(TOKEN_KEY) || undefined)
        .onConnect(onConnect)
        .onDisconnect(onDisconnect)
        .onConnectError(onConnectError);

    createSpacetimeDBProvider(connectionBuilder);

    const conn = useSpacetimeDB();

    const { children } = $props();
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

{#if $conn.isActive}
    <div class="grid h-dvh grid-cols-[auto_auto_1fr] gap-3 p-3">
        <ConversationList />
        <Separator orientation="vertical" />
        {@render children()}
    </div>
{/if}
