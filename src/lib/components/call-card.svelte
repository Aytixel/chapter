<script lang="ts">
    import { tables } from "$lib/module_bindings";
    import { CallFrameSource, type Call } from "$lib/module_bindings/types";
    import { useTable } from "spacetimedb/svelte";
    import Avatar from "./avatar.svelte";
    import { getUserUsername } from "$lib/user";
    import { useDecoder } from "$lib/call/decoder";
    import { createRawSnippet } from "svelte";
    import ContextMenu from "./ui/context-menu/context-menu.svelte";
    import {
        ContextMenuContent,
        ContextMenuItem,
        ContextMenuLabel,
        ContextMenuTrigger
    } from "./ui/context-menu";
    import { Mic, Monitor } from "@lucide/svelte";
    import { Slider } from "./ui/slider";
    import ContextMenuSeparator from "./ui/context-menu/context-menu-separator.svelte";
    import type { ClassValue } from "svelte/elements";
    import type { ContextMenuTriggerProps } from "bits-ui";
    import Badge from "./ui/badge/badge.svelte";

    const {
        call,
        class: classname,
        ...props
    }: { call: Call; class?: ClassValue | null } & ContextMenuTriggerProps = $props();

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

    const decoder = useDecoder();
    const camera_video = $derived($decoder?.getVideo(call.sender, CallFrameSource.Camera));
    const screen_video = $derived($decoder?.getVideo(call.sender, CallFrameSource.Screen));
    const camera_audio = $derived($decoder?.getAudio(call.sender, CallFrameSource.Camera));
    const screen_audio = $derived($decoder?.getAudio(call.sender, CallFrameSource.Screen));

    const renderElement = createRawSnippet<[Element]>((canvas) => ({
        render: () => "<div></div>",
        setup: (node) => node.append(canvas())
    }));
</script>

{#if $user[0]}
    <ContextMenu>
        <ContextMenuTrigger
            class={[
                "relative flex aspect-video h-48 min-h-0 min-w-0 items-center justify-center overflow-hidden rounded-md border bg-[color-mix(in_oklab,var(--muted)_15%,var(--background))] [&>div>canvas]:h-full [&>div>canvas]:w-full [&>div>canvas]:object-contain",
                classname
            ]}
            {...props}
        >
            {#if screen_video}
                {@render renderElement(screen_video)}
            {:else if camera_video}
                {@render renderElement(camera_video)}
            {:else}
                <Avatar src={avatar_url} alt={username} class="size-32 text-5xl" />
            {/if}
            <Badge
                class="absolute bottom-0 left-0 rounded-none rounded-tr-md bg-muted/50 px-4 text-sm text-foreground"
                >{username}</Badge
            >
        </ContextMenuTrigger>
        <ContextMenuContent class="w-xs" inert>
            <ContextMenuLabel>{username}</ContextMenuLabel>
            {#if camera_audio || screen_audio}
                <ContextMenuSeparator />
            {/if}
            {#if camera_audio}
                <ContextMenuItem>
                    <Mic />
                    <Slider
                        type="single"
                        bind:value={camera_audio.gain.value}
                        min={0}
                        max={1.5}
                        step={0.01}
                    />
                </ContextMenuItem>
            {/if}
            {#if screen_audio}
                <ContextMenuItem>
                    <Monitor />
                    <Slider
                        type="single"
                        bind:value={screen_audio.gain.value}
                        min={0}
                        max={1.5}
                        step={0.01}
                    />
                </ContextMenuItem>
            {/if}
        </ContextMenuContent>
    </ContextMenu>
{/if}
