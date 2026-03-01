import type { Group, User } from "./module_bindings/types";

export function getGroupName(group: Group, users_map: Map<string, User>): string {
    return (
        group.name ||
        group.users
            .map((user_identity) => users_map.get(user_identity.toString()))
            .filter((user) => user)
            .map((user) => user?.username || user?.identity.toString())
            .join(", ")
    );
}
