import type { User, UserStatus } from "./module_bindings/types";

export function getUserUsername(user: User, me: boolean = false): string {
    return (user.username || user.identity.toString()) + (me ? " (Vous)" : "");
}

export function getUsersMap(users: User[]): Map<string, User> {
    return new Map(users.map((user) => [user.identity.toString(), user]));
}

export function getUserStatus(status: UserStatus): { status: string; status_color: string } {
    return status.tag == "Online"
        ? {
              status: "Disponible",
              status_color: "bg-green-500"
          }
        : status.tag == "OnCall"
          ? {
                status: "En appel",
                status_color: "bg-red-500"
            }
          : status.tag == "Offline"
            ? {
                  status: "Déconnecter",
                  status_color: "bg-gray-500"
              }
            : {
                  status: "Inconnue",
                  status_color: "bg-background"
              };
}
