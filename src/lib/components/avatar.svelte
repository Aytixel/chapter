<script lang="ts">
    import type { ClassValue } from "svelte/elements";
    import { Avatar, AvatarFallback, AvatarImage } from "./ui/avatar";
    import { Label } from "./ui/label";

    const uid = $props.id();
    const {
        src,
        alt,
        class: classname,
        variant,
        onfile,
        ...props
    }: {
        src?: string;
        alt?: string;
        class?: ClassValue | null;
        variant?: "default" | "square";
        onfile?: (file?: File) => void;
    } = $props();
</script>

{#snippet avatar()}
    <Avatar {...props} class={["border", variant == "square" && "rounded-md", classname]}>
        <AvatarImage {src} {alt} class="object-cover" />
        <AvatarFallback class={[variant == "square" && "rounded-md"]}>
            {alt && alt.toString()[0].toUpperCase()}
        </AvatarFallback>
    </Avatar>
{/snippet}

{#if onfile}
    <Label class="cursor-pointer" for="{uid}-avatar-input">
        {@render avatar()}
    </Label>
    <input
        type="file"
        id="{uid}-avatar-input"
        class="hidden"
        accept="image/png, image/jpeg, image/webp, image/avif"
        onchange={(e) => onfile(e.currentTarget.files?.[0])}
    />
{:else}
    {@render avatar()}
{/if}
