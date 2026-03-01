<script lang="ts">
    import {
        Item,
        ItemMedia,
        ItemContent,
        ItemTitle,
        ItemDescription
    } from "$lib/components/ui/item";
    import { getUsername } from "$lib/user";
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

    const conn = useSpacetimeDB();
    const [users] = useTable(tables.user);
    const updateMessage = useReducer(reducers.updateMessage);
    const deleteMessage = useReducer(reducers.deleteMessage);

    const { message }: { message: Message } = $props();

    const me = $derived($conn.identity?.isEqual(message.sender));
    const user = $derived($users.find((user) => message.sender.isEqual(user.identity)));

    let modified_message = $state<string>();
</script>

{#if user}
    <ContextMenu>
        <ContextMenuTrigger class="w-full">
            <Item variant="outline" size="sm" class="p-1">
                <ItemMedia class="translate-0!">
                    <UserCard {user} variant="icon" />
                </ItemMedia>
                <ItemContent>
                    <ItemTitle class="text-xs">
                        {getUsername(user, me)}
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
                            placeholder="Envoyer un message..."
                            class="min-h-0"
                        />
                    {/if}
                </ItemContent>
            </Item>
        </ContextMenuTrigger>
        {#if me}
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
