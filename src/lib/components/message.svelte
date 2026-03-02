<script lang="ts">
    import {
        Item,
        ItemMedia,
        ItemContent,
        ItemTitle,
        ItemDescription,
        ItemHeader
    } from "$lib/components/ui/item";
    import { getUserUsername } from "$lib/user";
    import { Streamdown } from "svelte-streamdown";
    import StreamdownCode from "svelte-streamdown/code";
    import StreamdownMermaid from "svelte-streamdown/mermaid";
    import StreamdownMath from "svelte-streamdown/math";
    import UserCard from "./user-card.svelte";
    import { useReducer, useSpacetimeDB, useTable } from "spacetimedb/svelte";
    import type { Message } from "$lib/module_bindings/types";
    import { reducers, tables } from "$lib/module_bindings";
    import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";
    import {
        ContextMenu,
        ContextMenuTrigger,
        ContextMenuContent,
        ContextMenuItem
    } from "./ui/context-menu";
    import { SquarePen, Trash } from "@lucide/svelte";
    import { Textarea } from "./ui/textarea";

    const { message }: { message: Message } = $props();

    const conn = useSpacetimeDB();
    const [user] = useTable(tables.user.where((user) => user.identity.eq(message.sender)));
    const updateMessage = useReducer(reducers.updateMessage);
    const deleteMessage = useReducer(reducers.deleteMessage);

    const is_me = $derived($conn.identity?.isEqual(message.sender));

    let modified_message = $state<string>();
</script>

{#if $user[0]}
    <ContextMenu>
        <ContextMenuTrigger class="w-full">
            <Item variant="outline" size="sm" class="p-1">
                <ItemMedia class="translate-0!">
                    <UserCard user={$user[0]} variant="icon" />
                </ItemMedia>
                <ItemContent>
                    <ItemHeader>
                        <ItemTitle class="text-xs">
                            {getUserUsername($user[0], is_me)}
                        </ItemTitle>
                        <ItemDescription class="text-xs">
                            {#if message.createdAt.toMillis() == message.updatedAt.toMillis()}
                                {message.createdAt.toDate().toLocaleString()}
                            {:else}
                                <TooltipProvider>
                                    <Tooltip>
                                        <TooltipTrigger>
                                            {message.createdAt.toDate().toLocaleString()} (modifié)
                                        </TooltipTrigger>
                                        <TooltipContent>
                                            {message.updatedAt.toDate().toLocaleString()}
                                        </TooltipContent>
                                    </Tooltip>
                                </TooltipProvider>
                            {/if}
                        </ItemDescription>
                    </ItemHeader>
                    {#if modified_message === undefined}
                        <Streamdown
                            content={message.message}
                            components={{
                                code: StreamdownCode,
                                mermaid: StreamdownMermaid,
                                math: StreamdownMath
                            }}
                        />
                    {:else}
                        <Textarea
                            rows={Math.min(10, modified_message.split("\n").length)}
                            bind:value={modified_message}
                            onkeydown={(e) => {
                                if (e.key == "Enter" && !e.shiftKey) {
                                    updateMessage({
                                        messageId: message.id,
                                        message: modified_message || ""
                                    });
                                    modified_message = undefined;
                                }
                            }}
                            class="min-h-0"
                        />
                    {/if}
                </ItemContent>
            </Item>
        </ContextMenuTrigger>
        {#if is_me}
            <ContextMenuContent>
                <ContextMenuItem
                    class="flex justify-between"
                    onclick={() => (modified_message = message.message)}
                >
                    Modifier <SquarePen />
                </ContextMenuItem>
                <ContextMenuItem
                    class="flex justify-between"
                    variant="destructive"
                    onclick={() => deleteMessage({ messageId: message.id })}
                >
                    Supprimer <Trash />
                </ContextMenuItem>
            </ContextMenuContent>
        {/if}
    </ContextMenu>
{/if}
