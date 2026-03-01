<script lang="ts">
    import { Avatar, AvatarFallback, AvatarImage } from "./ui/avatar";
    import { Label } from "./ui/label";

    const uid = $props.id();
    const {
        src,
        alt,
        variant,
        onfile
    }: {
        src?: string;
        alt?: string;
        variant?: "default" | "square";
        onfile?: (file?: File) => void;
    } = $props();
</script>

{#snippet avatar()}
    <Avatar class={["border", variant == "square" && "rounded-md"]}>
        <AvatarImage {src} {alt} />
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
