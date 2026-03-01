import type { User } from "./module_bindings/types";

export function getUserUsername(user: User, me: boolean = false): string {
    return (user.username || user.identity.toString()) + (me ? " (Vous)" : "");
}

export function getUsersMap(users: User[]): Map<string, User> {
    return new Map(users.map((user) => [user.identity.toString(), user]));
}
