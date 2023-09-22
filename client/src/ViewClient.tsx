import { useLoaderData, useRevalidator } from "react-router-dom";
import { ErrorMessage } from "./components/ErrorMessage";
import { Result } from "./result";
import { Client } from "./bindings/Client";
import { Button, Callout, EditableText, Popover, Section, SectionCard } from "@blueprintjs/core";
import { css } from "@emotion/react";
import { Edit } from "@blueprintjs/icons";
import { useRef, useState } from "react";
import { updateClient } from "./api";

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

    const onClickEdit = () => {
      nameRef.current?.toggleEditing();
    }

    const commitClient = async (newClient: Object) => {
      return (await updateClient(newClient as Client))
        .andThen(async value => {
          revalidator.revalidate();
          return Result.Ok(value);
      });
    };

    return (
      <Section title="Details" rightElement={<Button onClick={onClickEdit} icon={<Edit />}></Button>}>
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

interface FieldEditorProps {
  field: string,
  original: Object,
  onSubmit: (object: Object) => Promise<Result<any>>;
}

function FieldEditor({ field, original, onSubmit }: FieldEditorProps) {
  const selfRef = useRef<EditableText>(null);

  const [updatedObject, setUpdatedObject] = useState<Object>(clone(original));
  const [errorMessage, setErrorMessage] = useState<string>("");

  return (
    <Popover
        enforceFocus={false}
        isOpen={errorMessage.length > 0}
        autoFocus={false}
        placement="bottom"
        content={<Callout intent="warning">{errorMessage}</Callout>}>

      <EditableText
          ref={selfRef}
          value={Reflect.get(updatedObject, field)}
          onChange={(value: string) => {
              setErrorMessage("");
              const nv = clone(updatedObject);
              Reflect.set(nv, field, value);
              setUpdatedObject(nv); }}
          onCancel={_ => {
            setUpdatedObject(original);
            setErrorMessage("");
          }}
          onConfirm={async (_: string) => {
            (await onSubmit(updatedObject)).match({
              ok: _ => {},
              err: msg => {
                setErrorMessage(msg);
                selfRef.current?.toggleEditing();
              }
            })
          }}
      />
    </Popover>
  );
}

function clone<T>(value: T) {
  return Object.assign(Object.create(Object.getPrototypeOf(value)), value) as T;
}