import { useLoaderData, useNavigate, useRevalidator } from "react-router-dom";
import { ErrorMessage } from "./components/ErrorMessage";
import { Result } from "./result";
import { Client } from "./bindings/Client";
import { Button, EditableText, Section, SectionCard } from "@blueprintjs/core";
import { css } from "@emotion/react";
import { Delete, Edit } from "@blueprintjs/icons";
import { useRef } from "react";
import { deleteClient, updateClient } from "./api";
import { FieldEditor } from "./components/FieldEditor";

export function ViewClient() {
  const client = useLoaderData() as Result<Client>;


  return client.match({
    ok: client => <div><Grid client={client} /></div>,
    err: msg => <ErrorMessage message={msg} />
  })
}

interface Props {
  client: Client
}

function Grid({ client }: Props) {
  function ClientDetails() {
    const nameRef = useRef<EditableText>(null);
    const revalidator = useRevalidator();
    const navigate = useNavigate();

    const onClickEdit = () => {
      nameRef.current?.toggleEditing();
    }

    const revalidate = async (value: Client) => {
      revalidator.revalidate();
      return Result.Ok(value);
    }

    const commitClient = async (newClient: Object) => {
      return (await updateClient(newClient as Client)).andThen(revalidate);
    };

    const navigateToClients = async (value: unknown) => {
      navigate("/");
      return Result.Ok(value);
    }

    const onClickDelete = async () => {
      return (await (await deleteClient(client)).andThen(revalidate)).andThen(navigateToClients);
    }

    return (
      <Section title="Details" rightElement={
        <>
          <Button onClick={onClickDelete} icon={<Delete />} />
          <Button onClick={onClickEdit} icon={<Edit />} />
        </>
        }>
        <SectionCard>
          <div css={css`display: grid; grid-template-columns: auto 1fr; grid-gap: 10px;`}>
            <span>Name:</span>
            <FieldEditor onSubmit={commitClient} field="name" original={client} />
            <span>IP&nbsp;Address:</span>
            <FieldEditor onSubmit={commitClient} field="ip" original={client} />
          </div>
        </SectionCard>
      </Section>
    );
  }

  return (
    <ClientDetails />
  )
}
