<script lang="ts">
    import type { Group, User } from "$lib/module_bindings/types";
    import { useTable } from "spacetimedb/svelte";
    import Avatar from "./avatar.svelte";
    import { Item, ItemContent, ItemMedia, ItemTitle } from "./ui/item";
    import { tables } from "$lib/module_bindings";
    import { getGroupName } from "$lib/group";
    import { getUsersMap } from "$lib/user";
    import EditGroupDialog from "./edit-group-dialog.svelte";

    const {
        group,
        variant,
        class: classname
    }: { group: Group; variant?: "default" | "icon"; class?: string } = $props();

    const [users] = useTable(tables.user);

    const users_map = $derived(getUsersMap($users as User[]));
    const name = $derived(getGroupName(group, users_map));
    const avatar_url = $derived(
        group.avatar &&
            URL.createObjectURL(
                new Blob([group.avatar as BlobPart], {
                    type: "image/webp"
                })
            )
    );
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
        <EditGroupDialog {group}>
            {#snippet child({ props })}
                <Avatar
                    {...props}
                    src={avatar_url}
                    alt={name}
                    variant="square"
                    class="cursor-pointer"
                />
            {/snippet}
        </EditGroupDialog>
    </ItemMedia>
    {#if variant != "icon"}
        <ItemContent class="w-full min-w-0 ">
            <ItemTitle
                class="block w-full cursor-[inherit] overflow-hidden text-nowrap text-ellipsis"
            >
                {name}
            </ItemTitle>
        </ItemContent>
    {/if}
</Item>
