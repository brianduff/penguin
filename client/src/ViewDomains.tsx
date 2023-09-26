import { useLoaderData } from "react-router-dom";
import { Result } from "./result";
import { DomainList } from "./bindings/DomainList";

export function ViewDomains() {
  const domains = useLoaderData() as Result<DomainList>;

  return <p>Hi</p>
}