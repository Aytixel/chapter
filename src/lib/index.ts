import type { ReceiverIdentity } from "./module_bindings/types";

export function compareReceivers(a: ReceiverIdentity, b: ReceiverIdentity): boolean {
    return a.tag == "User" && b.tag == "User"
        ? a.value.isEqual(b.value)
        : a.tag == "Group" && b.tag == "Group"
          ? a.value == b.value
          : false;
}
