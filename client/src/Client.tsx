import { useQuery } from "react-query";
import { useParams } from "react-router-dom";
import { getClient } from "./api";
import { ErrorMessage } from "./components/ErrorMessage";

export function Client() {
  const { id } = useParams();
  const query = useQuery("client", () => getClient(parseInt(id!)));
  if (query.isLoading) {
    return <></>
  }
  return query.data?.match({
    ok: client => <p>{client.name}</p>,
    err: message => <ErrorMessage message={message} />
  })
}