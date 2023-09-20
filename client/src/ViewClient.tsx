import { useLoaderData } from "react-router-dom";
import { ErrorMessage } from "./components/ErrorMessage";
import { Result } from "./result";
import { Client } from "./bindings/Client";

export function ViewClient() {
  const client = useLoaderData() as Result<Client>;

  return client.match({
    ok: client => <p>{client.name}</p>,
    err: msg => <ErrorMessage message={msg} />
  })
}

