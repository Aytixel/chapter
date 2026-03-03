<script lang="ts">
    import { Time } from "@internationalized/date";
    import { Timestamp } from "spacetimedb";

    const { timestamp }: { timestamp: Timestamp } = $props();

    // svelte-ignore state_referenced_locally
    let time = $state(new Time().add({ milliseconds: Timestamp.now().since(timestamp).millis }));
    const time_string = $derived(
        `${time.hour.toString().padStart(2, "0")}:${time.minute.toString().padStart(2, "0")}:${time.second.toString().padStart(2, "0")}`
    );

    setInterval(() => {
        time = new Time().add({ milliseconds: Timestamp.now().since(timestamp).millis });
    }, 1000);
</script>

<time datetime={time_string}>{time_string}</time>
