<script lang="ts">
    import "./layout.css";
    import favicon from "$lib/assets/favicon.svg";
    import { createSpacetimeDBProvider, useSpacetimeDB } from "spacetimedb/svelte";
    import { toSql, type Identity } from "spacetimedb";
    import { DbConnection, tables, type ErrorContext } from "$lib/module_bindings";
    import { Separator } from "$lib/components/ui/separator";
    import Sidebar from "$lib/components/sidebar.svelte";
    import { TooltipProvider } from "$lib/components/ui/tooltip";

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

    $effect(() => {
        const connection = $conn.getConnection();

        if (connection?.isActive) {
            connection.db[tables.call_frames.accessorName].onInsert((_, call_frame) => {
                console.log(call_frame);
            });
            connection.subscriptionBuilder().subscribe(toSql(tables.call_frames));
        }
    });

    const { children } = $props();
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

{#if $conn.isActive}
    <TooltipProvider>
        <div class="grid h-dvh grid-cols-[auto_auto_1fr] gap-3 p-3">
            <Sidebar />
            <Separator orientation="vertical" />
            {@render children()}
        </div>
    </TooltipProvider>
{/if}
