<script lang="ts">
    import type { User } from "$lib/module_bindings/types";
    import { useSpacetimeDB } from "spacetimedb/svelte";
    import { Item, ItemContent, ItemMedia, ItemTitle } from "./ui/item";
    import { getUserStatus, getUserUsername } from "$lib/user";
    import Avatar from "./avatar.svelte";
    import type { ClassValue } from "svelte/elements";
    import EditUserDialog from "./edit-user-dialog.svelte";
    import type { Snippet } from "svelte";

    const {
        user,
        variant,
        class: classname,
        child
    }: {
        user: User;
        variant?: "default" | "icon";
        class?: ClassValue | null;
        child?: Snippet<[]>;
    } = $props();

    const conn = useSpacetimeDB();

    const me = $derived($conn.identity?.isEqual(user.identity));
    const username = $derived(getUserUsername(user, me));
    const avatar_url = $derived(
        user.avatar &&
            URL.createObjectURL(
                new Blob([user.avatar as BlobPart], {
                    type: "image/webp"
                })
            )
    );

    const { status_color } = $derived(getUserStatus(user.status));
</script>

<Item
    class={[
        "cursor-default",
        variant == "icon" ? "w-fit p-0" : "grid grid-cols-[auto_1fr] p-1 transition-colors",
        classname
    ]}
    size="sm"
>
    <ItemMedia class="relative translate-0!">
        <EditUserDialog {user}>
            {#snippet child({ props })}
                <Avatar {...props} src={avatar_url} alt={username} />
                <div
                    class={["absolute right-0 bottom-0 size-2.5 rounded-full border", status_color]}
                ></div>
            {/snippet}
        </EditUserDialog>
    </ItemMedia>
    {#if variant != "icon"}
        <ItemContent class="grid w-full min-w-0 grid-cols-[1fr_auto] items-center gap-3">
            <ItemTitle
                class="block w-full cursor-[inherit] overflow-hidden text-nowrap text-ellipsis"
            >
                {username}
            </ItemTitle>
            {#if child}
                {@render child()}
            {/if}
        </ItemContent>
    {/if}
</Item>
