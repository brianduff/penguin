import { Client } from "./bindings/Client";

export async function getClients() {
  let result = await fetch("http://localhost:8080/api/v1/client");
  let json = await result.json();
  return json as Array<Client>;
}
