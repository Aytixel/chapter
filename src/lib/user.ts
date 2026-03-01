import type { User } from "./module_bindings/types";

export function getUsername(user: User, me: boolean = false) {
    return (user.username || user.identity.toString()) + (me ? " (Vous)" : "");
}
